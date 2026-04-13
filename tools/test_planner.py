#!/usr/bin/env python3
"""
Test Planning Agent

Reads a spec .md file from the specs/ directory, calls Claude to generate
a structured test plan, and writes the output to test-plans/.

Usage:
    python tools/test_planner.py specs/input.md
    python tools/test_planner.py specs/vector-graphics.md --output test-plans/vector-graphics.md
"""

import argparse
import sys
from pathlib import Path
import anthropic

SYSTEM_PROMPT = """\
You are a Test Planning Agent for a Rust game engine project (gnssyR).
Your job is to read a feature spec and produce a comprehensive, structured test plan.

The project uses spec-driven development:
  spec (.md) → test plan → tests (Rust) → implementation

The test engineer will implement your plan as runnable Rust test code.
Tests run headlessly using SoftwareDriver (CPU pixel buffer, no GPU) and
SimulatedBackend (injected input events, no hardware required).

## Output Format

Produce a Markdown document with the following structure:

```
# Test Plan: <Feature Name>

## Source Spec
<path to the spec file>

## Overview
<1–3 sentences summarising what is being tested and why>

## Test Cases

### TC-001: <Short descriptive name>
**Category:** unit | integration | property
**Priority:** high | medium | low
**Description:** <What this test verifies — one sentence>
**Setup:** <Any prerequisite state or construction needed>
**Steps:**
1. <action>
2. <action>
**Expected Result:** <What should be true after the steps>
**Notes:** <Edge cases, tolerance values, or implementation hints — omit if none>

### TC-002: ...
```

## Guidelines

- Cover the happy path, edge cases, and failure/error conditions
- For numeric comparisons (pixel positions, float values), specify tolerances explicitly
- Group related test cases; within a group, order simple → complex
- Prefer unit tests for pure logic, integration tests for driver/backend interactions
- If a test requires the SoftwareDriver, say so in Setup
- If a test requires the SimulatedBackend, say so in Setup
- Number test cases sequentially (TC-001, TC-002, …)
- Be specific: exact method names, parameter values, expected return values
- Do not invent behaviour that isn't in the spec; if the spec is ambiguous, note it
"""


def load_spec(path: Path) -> str:
    if not path.exists():
        print(f"error: spec file not found: {path}", file=sys.stderr)
        sys.exit(1)
    return path.read_text(encoding="utf-8")


def generate_test_plan(spec_path: Path, spec_content: str) -> str:
    client = anthropic.Anthropic()

    user_message = f"""\
Generate a complete test plan for the following spec.

Spec file: `{spec_path}`

---

{spec_content}"""

    print(f"Generating test plan for: {spec_path}", file=sys.stderr)
    print("Streaming response from claude-opus-4-6...", file=sys.stderr)

    collected_text: list[str] = []

    with client.messages.stream(
        model="claude-opus-4-6",
        max_tokens=16000,
        thinking={"type": "adaptive"},
        system=[
            {
                "type": "text",
                "text": SYSTEM_PROMPT,
                "cache_control": {"type": "ephemeral"},
            }
        ],
        messages=[
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": user_message,
                        "cache_control": {"type": "ephemeral"},
                    }
                ],
            }
        ],
    ) as stream:
        for event in stream:
            if (
                hasattr(event, "type")
                and event.type == "content_block_delta"
                and hasattr(event, "delta")
                and hasattr(event.delta, "type")
                and event.delta.type == "text_delta"
            ):
                chunk = event.delta.text
                collected_text.append(chunk)
                print(chunk, end="", flush=True)

        final = stream.get_final_message()

    print(file=sys.stderr)  # newline after streaming output

    usage = final.usage
    print(
        f"\nUsage — input: {usage.input_tokens} tokens "
        f"(cache_read: {getattr(usage, 'cache_read_input_tokens', 0)}, "
        f"cache_write: {getattr(usage, 'cache_creation_input_tokens', 0)}), "
        f"output: {usage.output_tokens} tokens",
        file=sys.stderr,
    )

    return "".join(collected_text)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate a structured test plan from a spec .md file."
    )
    parser.add_argument("spec", type=Path, help="Path to the spec .md file")
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=None,
        help="Output path for the test plan (default: test-plans/<spec-name>.md)",
    )
    args = parser.parse_args()

    spec_path: Path = args.spec
    spec_content = load_spec(spec_path)

    if args.output is not None:
        output_path: Path = args.output
    else:
        test_plans_dir = spec_path.parent.parent / "test-plans"
        test_plans_dir.mkdir(exist_ok=True)
        output_path = test_plans_dir / spec_path.name

    plan = generate_test_plan(spec_path, spec_content)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(plan, encoding="utf-8")
    print(f"\nTest plan written to: {output_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
