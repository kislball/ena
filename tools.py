#!/usr/bin/python
import toml
import argparse
import os

program = argparse.ArgumentParser()

program.add_argument("--dry", type=bool, help="Only print what is going to be done", default=False)
program.add_argument("--set-version", type=bool, default=False, help="Sets version")
program.add_argument("--publish", type=bool, default=False, help="Publishes packages")
program.add_argument("--tag", type=bool, default=False, help="Creates a tag with current version in git")

parsed = program.parse_args()

VERSION = open("./version.txt").read()
PUBLISHING_ORDER = open("./publish-order.txt").read().splitlines()

if parsed.dry:
    print("running in dry mode")

def tag():
    run(f"""git commit -a -m "v{VERSION}" """)
    run(f"git tag v{VERSION}")

def run(command):
    print(f"running {command}")
    if not parsed.dry:
        os.system(command)

def get_tomls():
    data = toml.load("./Cargo.toml")
    packages = []

    for member in data["workspace"]["members"]:
        packages.append(f"{member}/Cargo.toml")

    return packages

def set_version(file: str):
    data = toml.load(file)
    data["package"]["version"] = VERSION
    for dependency in data["dependencies"]:
        if dependency.startswith("enalang"):
            data["dependencies"][dependency]["version"] = VERSION
    if not parsed.dry:
        toml.dump(data, open(file, "w"), encoder=toml.TomlArraySeparatorEncoder(preserve=True, separator=""))

def set_versions():
    for toml in get_tomls():
        print(f"setting version v{VERSION} for {toml}")
        set_version(toml)

def publish():
    for package in PUBLISHING_ORDER:
        print(f"publishing {package}")
        run(f"cargo publish -p {package}")

def main():
    if parsed.set_version:
        set_versions()
    
    if parsed.tag:
        tag()

    if parsed.publish:
        publish()
    
    if not parsed.tag and not parsed.publish and not parsed.set_version:
        print("nothing to do :(")

if __name__ == "__main__":
    main()