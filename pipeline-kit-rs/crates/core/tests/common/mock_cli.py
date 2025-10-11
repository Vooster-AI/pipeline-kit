#!/usr/bin/env python3
"""Mock CLI that outputs predefined JSON Lines for testing CliExecutor."""

import json
import sys
import time

def main():
    # Output predefined JSON Lines
    events = [
        {"type": "system", "message": "Starting mock CLI"},
        {"type": "assistant", "content": [{"type": "text", "text": "Hello from mock"}]},
        {"type": "tool_call", "name": "Read", "input": {"file_path": "/test.txt"}},
        {"type": "assistant", "content": [{"type": "text", "text": "Processing complete"}]},
        {"type": "result", "session_id": "mock-session-123", "duration_ms": 100}
    ]

    for event in events:
        print(json.dumps(event), flush=True)
        # Small delay to simulate streaming
        time.sleep(0.01)

if __name__ == "__main__":
    main()
