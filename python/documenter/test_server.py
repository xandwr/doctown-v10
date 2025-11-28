#!/usr/bin/env python3
"""
Simple test script for the documenter server.

Run the server first with: python server.py
Then run this script: python test_server.py
"""

import requests
import json

BASE_URL = "http://localhost:18116"

def test_health():
    """Test health check endpoint."""
    print("Testing /health endpoint...")
    response = requests.get(f"{BASE_URL}/health")
    print(f"Status: {response.status_code}")
    print(f"Response: {json.dumps(response.json(), indent=2)}\n")
    return response.status_code == 200

def test_models():
    """Test models listing endpoint."""
    print("Testing /models endpoint...")
    response = requests.get(f"{BASE_URL}/models")
    print(f"Status: {response.status_code}")
    models = response.json()
    print(f"Available models: {list(models['models'].keys())}\n")
    return response.status_code == 200

def test_summarize():
    """Test summarization endpoint."""
    print("Testing /summarize endpoint...")

    test_code = """
def calculate_fibonacci(n):
    if n <= 1:
        return n
    return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)
"""

    payload = {
        "text": test_code,
        "instructions": "Summarize this function in 1-2 sentences."
    }

    print("Sending code to summarize...")
    response = requests.post(
        f"{BASE_URL}/summarize",
        json=payload,
        timeout=60
    )

    print(f"Status: {response.status_code}")

    if response.status_code == 200:
        result = response.json()
        print(f"Summary: {result['summary']}\n")
        return True
    else:
        print(f"Error: {response.text}\n")
        return False

def main():
    """Run all tests."""
    print("=" * 60)
    print("Doctown Summarizer - Test Suite")
    print("=" * 60 + "\n")

    tests = [
        ("Health Check", test_health),
        ("Models List", test_models),
        ("Summarization", test_summarize),
    ]

    results = []
    for name, test_func in tests:
        try:
            success = test_func()
            results.append((name, success))
        except Exception as e:
            print(f"ERROR in {name}: {e}\n")
            results.append((name, False))

    print("=" * 60)
    print("Test Results:")
    print("=" * 60)

    for name, success in results:
        status = "✓ PASS" if success else "✗ FAIL"
        print(f"{status} - {name}")

    all_passed = all(success for _, success in results)

    if all_passed:
        print("\n🎉 All tests passed!")
    else:
        print("\n⚠️  Some tests failed. Check server logs.")

    return 0 if all_passed else 1

if __name__ == "__main__":
    exit(main())
