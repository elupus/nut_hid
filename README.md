# NUT HID

NUT HID is a Windows User-Mode Driver Framework (UMDF) HID driver and utilities for exposing [Network UPS Tools (NUT)](https://networkupstools.org/) devices as HID devices on Windows. This allows Windows to interact with UPS devices managed by NUT as if they were native HID UPS devices.

## Features

- UMDF HID driver for Windows
- Supports multiple backends:
  - NUT network backend (connects to a NUT server)
  - Dummy backend (for testing)
  - Mini backend (minimal implementation)
- CLI utility for creating and managing virtual HID devices
- Device property configuration via CLI and INF

## Project Structure

```
nut_hid/                # Main driver crate (UMDF HID driver)
nut_hid_device/         # Device backend implementations (NUT, dummy, mini)
nut_hid_cli/            # CLI utility for device creation and property management
scripts/                # Deployment and installation scripts
src/                    # Driver source code
```

## Building

1. **Prerequisites:**
   - Rust nightly toolchain
   - Windows Driver Kit (WDK)
   - [windows-drivers-rs](https://github.com/microsoft/windows-drivers-rs) dependencies

2. **Build the driver and CLI:**


   Ensure you have activate a WDK build shell then:
   ```bat
   cargo make
   ```

## Installation


2. **Install the driver:**

   ```bat
   pnputil.exe /add-driver nut_hid.inf /install
   ```

3. **Add a virtual device:**

   ```bat
   nut_hid_cli --backend nut --host <NUT_HOST> --port <NUT_PORT>
   ```

## Usage

- The CLI utility (`nut_hid_cli`) can be used to create a virtual HID device with configurable properties.
- The driver will communicate with the specified backend (e.g., a NUT server) and expose UPS information to Windows as a HID device.

## Configuration

- Device properties such as backend, host, and port can be set via CLI arguments

## License

This project is licensed under the [Apache License 2.0](LICENSE).

## Credits

- Joakim Plate
- [Network UPS Tools (NUT)](https://networkupstools.org/)
- [windows-drivers-rs](https://github.com/microsoft/windows-drivers-rs)

---