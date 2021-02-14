# frozen_string_literal: true

require 'changelogerator'
require 'git'
require 'erb'
require 'toml'
require 'json'
require_relative './lib.rb'

version = ENV['GITHUB_REF']
token = ENV['GITHUB_TOKEN']

shadows_path = ENV['GITHUB_WORKSPACE'] + '/shadows/'
pg = Git.open(shadows_path)

# Generate an ERB renderer based on the template .erb file
renderer = ERB.new(
  File.read(ENV['GITHUB_WORKSPACE'] + '/shadows/scripts/github/shadows_release.erb'),
  trim_mode: '<>'
)

# get last shadows version. Use handy Gem::Version for sorting by version
last_version = pg
              .tags
              .map(&:name)
              .grep(/^v\d+\.\d+\.\d+.*$/)
              .sort_by { |v| Gem::Version.new(v.slice(1...)) }[-2]

shadows_cl = Changelog.new(
  'shadowsnetwork/shadows', last_version, version, token: token
)

# Get prev and cur substrate SHAs - parse the old and current Cargo.lock for
# shadows and extract the sha that way.
prev_cargo = TOML::Parser.new(pg.show("#{last_version}:Cargo.lock")).parsed
current_cargo = TOML::Parser.new(pg.show("#{version}:Cargo.lock")).parsed

substrate_prev_sha = "v" + prev_cargo['package']
                    .find { |p| p['name'] == 'sp-runtime' }['version']

substrate_cur_sha = "v" + current_cargo['package']
                    .find { |p| p['name'] == 'sp-runtime' }['version']

substrate_cl = Changelog.new(
  'paritytech/substrate', substrate_prev_sha, substrate_cur_sha,
  token: token,
  prefix: true
)

all_changes = shadows_cl.changes + substrate_cl.changes

# Set all the variables needed for a release

misc_changes = Changelog.changes_with_label(all_changes, 'B1-releasenotes')
client_changes = Changelog.changes_with_label(all_changes, 'B5-clientnoteworthy')
runtime_changes = Changelog.changes_with_label(all_changes, 'B7-runtimenoteworthy')

release_priority = Changelog.highest_priority_for_changes(all_changes)

# Pulled from the previous Github step
rustc = ENV['RUSTC']
toolchain_nightly = ENV['WASM_BUILD_TOOLCHAIN']

shadows_runtime = get_runtime('parachain.rs', shadows_path)

# These json files should have been downloaded as part of the build-runtimes
# github action

puts renderer.result
