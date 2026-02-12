"""
Voice processing server for Days Tracker Kiosk.

Receives WAV audio from the ESP32, processes it through:
1. Groq Whisper API (speech-to-text)
2. Groq LLM (intent parsing + structured JSON action)

Returns a JSON action for the ESP32 to apply.

Usage:
    pip install fastapi uvicorn httpx
    export GROQ_API_KEY="your-key-here"
    uvicorn voice_server:app --host 0.0.0.0 --port 8000
"""

import json
import logging
import os
from io import BytesIO
from typing import Optional
from urllib.parse import unquote_plus

import httpx
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="Days Tracker Voice Server")

GROQ_API_KEY = os.environ.get("GROQ_API_KEY", "")
GROQ_API_BASE = "https://api.groq.com/openai/v1"

# Groq models
STT_MODEL = "whisper-large-v3-turbo"
LLM_MODEL = "llama-3.3-70b-versatile"


@app.post("/voice")
async def process_voice(request: Request):
    """
    Receive WAV audio from ESP32, transcribe with Whisper, parse intent with LLM.

    Query params:
        context: URL-encoded string describing current tasks

    Body:
        Raw WAV audio bytes (Content-Type: audio/wav)

    Returns:
        JSON with action, task_name, recurrence_days, message, task_id
    """
    if not GROQ_API_KEY:
        return JSONResponse(
            status_code=500,
            content={"action": "none", "message": "Server error: GROQ_API_KEY not set"},
        )

    # Get task context from query params
    context = request.query_params.get("context", "No tasks exist yet.")
    context = unquote_plus(context)

    # Read WAV body
    audio_data = await request.body()
    logger.info(f"Received {len(audio_data)} bytes of audio")

    if len(audio_data) < 100:
        return JSONResponse(
            content={"action": "none", "message": "Audio too short", "task_name": ""},
        )

    # Step 1: Speech-to-text via Groq Whisper
    transcript = await transcribe_audio(audio_data)
    if not transcript:
        return JSONResponse(
            content={"action": "none", "message": "Could not understand audio", "task_name": ""},
        )

    logger.info(f"Transcript: {transcript}")

    # Step 2: Intent parsing via Groq LLM
    action = await parse_intent(transcript, context)
    logger.info(f"Action: {action}")

    return JSONResponse(content=action)


async def transcribe_audio(audio_data: bytes) -> Optional[str]:
    """Send audio to Groq Whisper API for transcription."""
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{GROQ_API_BASE}/audio/transcriptions",
                headers={"Authorization": f"Bearer {GROQ_API_KEY}"},
                files={"file": ("audio.wav", BytesIO(audio_data), "audio/wav")},
                data={"model": STT_MODEL, "language": "en"},
            )
            response.raise_for_status()
            result = response.json()
            return result.get("text", "").strip()
    except Exception as e:
        logger.error(f"Whisper API error: {e}")
        return None


async def parse_intent(transcript: str, task_context: str) -> dict:
    """Parse voice command transcript into a structured action using Groq LLM."""

    system_prompt = """You are a task management assistant for a "Days Tracker" device.
The user speaks voice commands to manage recurring tasks.

Given the user's spoken command and the current task list, output ONLY a JSON object with these fields:
- "action": one of "create", "complete", "update", "delete", or "none"
- "task_name": the task name (string)
- "recurrence_days": number of days between recurrences (integer, e.g. 3 for every 3 days, 7 for weekly, 30 for monthly)
- "message": a short confirmation message to display on the device screen (max 60 chars)
- "task_id": the ID of an existing task to act on (integer or null for create/none)

Rules:
- For "create": set task_name and recurrence_days. task_id should be null.
- For "complete": match the spoken task name to an existing task. Set task_id if you can match it.
- For "update": set task_id and any fields to change.
- For "delete": set task_id of the task to remove.
- For "none": when the command doesn't map to any task action.
- If the user says something like "water plants every 3 days", create with recurrence_days=3.
- If the user says "weekly", use recurrence_days=7. "Monthly" = 30. "Daily" = 1.
- Match task names fuzzy - "water plants" should match "Water plants" or "water the plants".

Output ONLY valid JSON, no markdown, no explanation."""

    user_prompt = f"""Task context: {task_context}

Voice command: "{transcript}"

Output the JSON action:"""

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{GROQ_API_BASE}/chat/completions",
                headers={
                    "Authorization": f"Bearer {GROQ_API_KEY}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": LLM_MODEL,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": user_prompt},
                    ],
                    "temperature": 0.1,
                    "max_tokens": 256,
                },
            )
            response.raise_for_status()
            result = response.json()
            content = result["choices"][0]["message"]["content"].strip()

            # Strip markdown code fences if present
            if content.startswith("```"):
                content = content.split("\n", 1)[-1]
                if content.endswith("```"):
                    content = content[:-3]
                content = content.strip()

            action = json.loads(content)

            # Ensure all required fields exist
            action.setdefault("action", "none")
            action.setdefault("task_name", "")
            action.setdefault("recurrence_days", None)
            action.setdefault("message", "Command processed")
            action.setdefault("task_id", None)

            return action

    except json.JSONDecodeError as e:
        logger.error(f"LLM returned invalid JSON: {e}")
        return {
            "action": "none",
            "task_name": "",
            "message": f"Heard: {transcript[:40]}",
            "task_id": None,
        }
    except Exception as e:
        logger.error(f"LLM API error: {e}")
        return {
            "action": "none",
            "task_name": "",
            "message": "Server processing error",
            "task_id": None,
        }


@app.get("/health")
async def health():
    """Health check endpoint."""
    return {
        "status": "ok",
        "groq_key_set": bool(GROQ_API_KEY),
        "stt_model": STT_MODEL,
        "llm_model": LLM_MODEL,
    }
