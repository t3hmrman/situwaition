git := env_var_or_default("GIT", "git")
rustc := env_var_or_default("RUSTC", "rustc")
cargo := env_var_or_default("CARGO", "cargo")
cargo_watch := env_var_or_default("CARGO_WATCH", "cargo-watch")

just := env_var_or_default("JUST", just_executable())

root_dir := invocation_directory()

version := `{{cargo}} get version | head --bytes=-1`
sha := `{{git}} rev-parse --short HEAD`

default:
  {{just}} --list

#############
# Utilities #
#############

# Print the current version
print-version:
    @echo -n "{{version}}"

# Print the current SHA
print-sha:
    @echo -n "{{sha}}"

# Ensure a binary is present
ensure-binary bin env_name:
    #/usr/bin/env -S bash -euo pipefail
    if [ -z "$(command -v {{bin}})" ]; then
    echo "Missing binary [{{bin}}], make sure it is installed & on your PATH (or override it's location with {{env_name}})";
    exit 1;
    fi

###########
# Project #
###########

# Set up the project
setup:
    @{{just}} ensure-binary rustc RUSTC
    @{{just}} ensure-binary cargo CARGO
    @{{just}} ensure-binary cargo-watch CARGO_WATCH

# Format
fmt:
    {{cargo}} fmt

# Lint
lint:
    {{cargo}} clippy

# Lint the project
lint-watch:
    @{{cargo}} watch --watch=src --shell 'just lint'

# Build
build:
    {{cargo}} build

# Build continuously (development mode)
build-watch:
    {{cargo}} watch -x build

# Build the release version of the binary
build-release:
    @{{cargo}} build --release

# Check the project
check:
    @{{cargo}} check

# Ensure that the # of commits is what we expect
# NOTE: we can't write a simpler script here because some CI environments don't have bash in the same place
# and/or are using busybox env (which doesn't support -S)
check-commit-count now before count:
    @export COUNT=$(($(git rev-list --count {{now}} --no-merges) - $(git rev-list --count {{before}}))) && \
    if [ "$COUNT" != "1" ]; then \
      echo -e "[error] number of commits ($COUNT) is *not* {{count}} -- please squash commits"; \
      exit 1; \
    fi

######################
# Release Management #
######################

changelog_file_path := env_var_or_default("CHANGELOG_FILE_PATH", "CHANGELOG")

# Generate the changelog
changelog:
  {{git}} cliff --unreleased --tag={{version}} --prepend={{changelog_file_path}}

release-major:
    {{git}} fetch --tags
    {{cargo}} set-version --bump major
    {{just}} changelog
    {{git}} commit -am "release: v`just print-version`"
    {{git}} push

release-minor:
    {{git}} fetch --tags
    {{cargo}} set-version --bump minor
    {{just}} changelog
    {{git}} commit -am "release: v`just print-version`"
    {{git}} push

release-patch:
    {{git}} fetch --tags
    {{cargo}} set-version --bump patch
    {{just}} changelog
    {{git}} commit -am "release: v`just print-version`"
    {{git}} push