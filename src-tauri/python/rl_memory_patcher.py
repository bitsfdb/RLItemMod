
import sys
import time

try:
    import pymem
except ImportError:
    print("Error: pymem is required. Please run 'pip install pymem'")
    sys.exit(1)

def scan_and_replace(pm: pymem.Pymem, target: bytes, replacement: bytes):
    if len(replacement) > len(target):
        print(f"[!] Error: Replacement string '{replacement.decode()}' ({len(replacement)} bytes) is longer than target '{target.decode()}' ({len(target)} bytes).")
        print("    In-place memory replacement cannot safely expand strings without corrupting memory pools.")
        return False

    padded_replacement = replacement.ljust(len(target), b'\x00')
    search_bytes = target + b'\x00'
    padded_replacement_with_null = padded_replacement + b'\x00'

    print(f"[*] Scanning memory for: {target.decode()}...")
    count = 0
    system_info = pymem.process.get_system_info()
    min_address = system_info.lpMinimumApplicationAddress
    max_address = system_info.lpMaximumApplicationAddress

    address = min_address
    while address < max_address:
        mbi = pymem.memory.virtual_query(pm.process_handle, address)
        if mbi.State == 0x1000 and (mbi.Protect == 0x04 or mbi.Protect == 0x40):
            try:
                buffer = pm.read_bytes(mbi.BaseAddress, mbi.RegionSize)
                offset = 0
                while True:
                    offset = buffer.find(search_bytes, offset)
                    if offset == -1:
                        break
                    match_address = mbi.BaseAddress + offset
                    pm.write_bytes(match_address, padded_replacement_with_null, len(padded_replacement_with_null))
                    print(f"[+] Replaced occurrence at 0x{match_address:X}")
                    count += 1
                    offset += len(search_bytes)
            except pymem.exception.MemoryReadError:
                pass
        address += mbi.RegionSize

    print(f"[*] Done. Replaced {count} instances.")
    return count > 0

def main():
    if len(sys.argv) != 3:
        print("Usage: python rl_memory_patcher.py <OriginalItem> <ReplacementItem>")
        print("Example: python rl_memory_patcher.py Flamethrower AlphaReward")
        sys.exit(1)

    original_item = sys.argv[1].encode('ascii')
    replacement_item = sys.argv[2].encode('ascii')

    print("[*] Connecting to RocketLeague.exe...")
    try:
        pm = pymem.Pymem("RocketLeague.exe")
    except pymem.exception.ProcessNotFound:
        print("[!] RocketLeague.exe is not currently running. Please launch the game first.")
        sys.exit(1)
    print(f"[+] Connected to process (PID: {pm.process_id})")
    start_time = time.time()
    scan_and_replace(pm, original_item, replacement_item)
    elapsed = time.time() - start_time
    print(f"[*] Memory patch operation completed in {elapsed:.2f} seconds.")

if __name__ == "__main__":
    main()