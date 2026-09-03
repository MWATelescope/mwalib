import subprocess
import sys
import toml
import json
import re


def get_msrv(path_to_cargo_toml: str):
    # Read rust-version from Cargo.toml
    try:
        cargo_toml = toml.load(path_to_cargo_toml)
        msrv = cargo_toml.get("package", {}).get("rust-version")
        if msrv:
            return msrv
    except Exception:
        pass

    # Fallback: use current rustc version
    rustc_version = subprocess.check_output(["rustc", "--version"]).decode().strip()
    match = re.search(r"(\d+\.\d+\.\d+)", rustc_version)
    return match.group(1) if match else None


def get_dependencies():
    # Run cargo metadata
    output = subprocess.check_output(["cargo", "metadata", "--format-version", "1"])
    metadata = json.loads(output)
    return metadata["packages"]


def compare_versions(v1, v2):
    # Compare semantic versions (returns True if v1 > v2)
    def parse(v):
        return tuple(map(int, v.split(".")))

    return parse(v1) > parse(v2)


def main():
    msrv = get_msrv(sys.argv[1])
    if not msrv:
        print("Could not determine MSRV.")
        return

    print(f"MSRV: {msrv}\nChecking dependencies...\n")
    packages = get_dependencies()
    issues = []

    for pkg in packages:
        name = pkg["name"]
        version = pkg["version"]
        rust_version = pkg.get("rust_version")
        if rust_version and compare_versions(rust_version, msrv):
            issues.append((name, version, rust_version))

    if issues:
        print("❌ The following crates require a higher Rust version than MSRV:")
        for name, version, rust_version in issues:
            print(f"  - {name}@{version} requires Rust {rust_version} (MSRV is {msrv})")
    else:
        print("✅ All dependencies are compatible with MSRV.")


if __name__ == "__main__":
    main()
