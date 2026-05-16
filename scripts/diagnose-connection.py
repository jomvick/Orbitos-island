#!/usr/bin/env python3
import socket
import os
import sys
import stat

SOCKET_PATH = os.path.expanduser("~/.agentosd.sock")

def check_socket():
    print(f"--- Checking socket: {SOCKET_PATH} ---")
    
    if not os.path.exists(SOCKET_PATH):
        print(f"ERROR: Socket file does not exist at {SOCKET_PATH}")
        return False
        
    s = os.stat(SOCKET_PATH)
    is_socket = stat.S_ISSOCK(s.st_mode)
    print(f"Is it a socket? {'Yes' if is_socket else 'No'}")
    
    permissions = oct(s.st_mode & 0o777)
    print(f"Permissions: {permissions}")
    print(f"Owner UID: {s.st_uid}")
    print(f"Current UID: {os.getuid()}")
    
    if not is_socket:
        print("ERROR: File exists but is NOT a socket.")
        return False
        
    try:
        print("Attempting to connect...")
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.settimeout(2)
        client.connect(SOCKET_PATH)
        print("SUCCESS: Connected to socket!")
        
        # Try a ping command
        import json
        ping = {
            "type": "command",
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "command": {"action": "ping"},
            "timestamp": "2026-05-15T21:00:00Z"
        }
        client.sendall(json.dumps(ping).encode() + b"\n")
        response = client.recv(1024)
        print(f"Response from daemon: {response.decode().strip()}")
        
        client.close()
        return True
    except Exception as e:
        print(f"ERROR: Failed to connect or communicate: {e}")
        return False

if __name__ == "__main__":
    success = check_socket()
    if not success:
        sys.exit(1)
    print("\nAll checks passed from Python's perspective.")
