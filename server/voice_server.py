"""
Voice processing server for Days Tracker Kiosk.

Receives WAV audio from the ESP32, processes it through:
1. Groq Whisper API (speech-to-text)
2. Groq LLM (extract task name and recurrence from speech)

Returns a JSON action for the ESP32 to create a new task.

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
    Receive WAV audio from ESP32, transcribe with Whisper, extract task details with LLM.

    Body:
        Raw WAV audio bytes (Content-Type: audio/wav)

    Returns:
        JSON with action ("create" or "none"), task_name, recurrence_days, message
    """
    if not GROQ_API_KEY:
        return JSONResponse(
            status_code=500,
            content={"action": "none", "message": "Server error: GROQ_API_KEY not set"},
        )

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

    # Step 2: Extract task name and recurrence via Groq LLM
    action = await extract_task(transcript)
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


async def extract_task(transcript: str) -> dict:
    """Extract task name and recurrence from the transcribed voice command."""

    system_prompt = """You extract task information from spoken commands for a recurring task tracker.

Given the user's speech, output ONLY a JSON object with these fields:
- "action": "create" if a task can be extracted, otherwise "none"
- "task_name": a short, clear task name (e.g. "Water plants", "Take vitamins")
- "recurrence_days": how often in days (1=daily, 7=weekly, 14=biweekly, 30=monthly, 365=yearly)
- "message": a confirmation message for the device screen (max 60 chars)

Examples:
- "Water the plants every three days" → {"action":"create","task_name":"Water plants","recurrence_days":3,"message":"Water plants every 3 days"}
- "Remind me to take vitamins daily" → {"action":"create","task_name":"Take vitamins","recurrence_days":1,"message":"Take vitamins daily"}
- "Change air filter monthly" → {"action":"create","task_name":"Change air filter","recurrence_days":30,"message":"Change air filter monthly"}
- "hello how are you" → {"action":"none","task_name":"","recurrence_days":null,"message":"No task found in speech"}

If no recurrence is mentioned, default to 1 (daily).
Keep task names concise (2-5 words). Capitalize the first letter.
Output ONLY valid JSON, no markdown, no explanation."""

    user_prompt = f'"{transcript}"'

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
                    "max_tokens": 200,
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

            return action

    except json.JSONDecodeError as e:
        logger.error(f"LLM returned invalid JSON: {e}")
        return {
            "action": "none",
            "task_name": "",
            "message": f"Heard: {transcript[:40]}",
        }
    except Exception as e:
        logger.error(f"LLM API error: {e}")
        return {
            "action": "none",
            "task_name": "",
            "message": "Server processing error",
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
