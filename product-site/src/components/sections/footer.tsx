"use client";

import { Button } from "../ui/button";

export function Footer() {
  return (
    <footer className="bg-[var(--background)] border-t border-[var(--border)]">
      <div className="max-w-6xl mx-auto px-6 py-12">
        {/* Newsletter + Logo row */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 mb-8">
          {/* Logo */}
          <a href="#" className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-[var(--accent)] flex items-center justify-center">
              <span className="text-white font-serif text-lg font-bold">D</span>
            </div>
            <span className="text-lg font-semibold text-[var(--foreground)]">
              DaysTracker
            </span>
          </a>

          {/* Newsletter */}
          <div className="flex gap-2">
            <input
              type="email"
              placeholder="Enter your email"
              className="px-4 py-2 rounded-lg border border-[var(--border)] bg-transparent text-sm focus:outline-none focus:ring-2 focus:ring-[var(--accent)] w-full md:w-64"
            />
            <Button size="sm">Subscribe</Button>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="flex flex-col md:flex-row justify-between items-center gap-4 pt-8 border-t border-[var(--border)]">
          <p className="text-[var(--muted)] text-sm">
            &copy; {new Date().getFullYear()} DaysTracker. All rights reserved.
          </p>
          <div className="flex items-center gap-6 text-sm text-[var(--muted)]">
            <a href="#" className="hover:text-[var(--foreground)] transition-colors">Privacy</a>
            <a href="#" className="hover:text-[var(--foreground)] transition-colors">Terms</a>
            <a href="#" className="hover:text-[var(--foreground)] transition-colors">Contact</a>
          </div>
        </div>
      </div>
    </footer>
  );
}
