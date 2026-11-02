# Command Reference

Kodama commands accept the usual `--help` flag. Most commands also have visible aliases for shorter interactive use.

## `kodama new`

Creates site files, config files, or sections.

```sh
kodama new site <path>
kodama new config [path]
kodama new post <path>
```

Aliases:

- `kodama n`
- `kodama new s`
- `kodama new c`
- `kodama new p`

### `new site`

Creates a new directory and writes a default Kodama project into it.

Options:

- `--no-typst`, alias `--nt`: skip Typst library files and create a Markdown starter section.

### `new config`

Writes a default configuration file. The default path is `Kodama.toml`.

### `new post`

Creates a new section under the configured source tree.

Options:

- `--format typ` or `--format md`, short `-f`: section format used when the path has no extension. The file is created with the configured `[suffix]` extension for that format. Defaults to `typ`.
- `--template <path>`, short `-t`: template file. The placeholder `<FILE_NAME>` is replaced with the new file stem.
- `--config <path>`, short `-c`: configuration file.

## `kodama init`

```sh
kodama init [path]
```

Initializes an existing directory as a Kodama project. The directory must already exist.

Options:

- `--no-typst`, alias `--nt`: skip Typst library files.

Alias: `kodama i`.

## `kodama build`

```sh
kodama build
```

Compiles the current site into the publish output directory.

Options:

- `--config <path>`, short `-c`: configuration file.
- `--free`: compile documents in place under the current directory without reading configuration (see [Free mode](#free-mode)).
- `--verbose`, short `-v`: print build output.
- `--verbose-skip`: print skip output.
- `--no-cache`, alias `--nc`: rebuild all files without using caches.
- `--indexes`: generate `kodama.json`.
- `--no-indexes`: skip `kodama.json`.
- `--graph`: generate `kodama.graph.json`.
- `--no-graph`: skip `kodama.graph.json`.

Alias: `kodama b`.

## `kodama check`

```sh
kodama check
```

Validates the site without generating build artifacts.

Options:

- `--config <path>`, short `-c`: configuration file.
- `--free`: check documents under the current directory without reading configuration.
- `--strict`: treat warnings as errors.

Alias: `kodama c`.

## `kodama serve`

```sh
kodama serve
```

Builds the site into the serve output directory, starts a static server (built-in unless `[serve].command` selects an external one, or skipped with `--no-server`), watches source/config/theme/assets files, and rebuilds on changes.

The built-in server listens on `127.0.0.1`; set `[serve].port` to pin its port, otherwise it uses the first free port from 8080.

Options:

- `--config <path>`, short `-c`: configuration file.
- `--free`: compile and serve documents in place under the current directory without reading configuration.
- `--verbose`, short `-v`: print build output.
- `--verbose-skip`: print skip output.
- `--disable-reload`, short `-d`: disable live reload.
- `--watch-stats`, short `-w`: print dirty-path analysis for each watch batch.
- `--no-server`: build and watch the site without starting any static server. Useful for headless watch-only workflows.
- `--print-json`: emit line-delimited JSON events (`ready`, `rebuilt`) on stdout for editor and tooling integrations. Requires `--no-server` so stdout stays machine-readable; terminal escape sequences and watch status messages are suppressed while active.
- `--indexes`: generate `kodama.json` during serve.
- `--no-indexes`: skip `kodama.json`.
- `--graph`: generate `kodama.graph.json` during serve.
- `--no-graph`: skip `kodama.graph.json`.

Alias: `kodama s`.

`--print-json` requires `--no-server`; running `--print-json` without `--no-server` is rejected.

## Free mode

`build --free`, `serve --free`, and `check --free` use the current directory as the source root. `--free` conflicts with `--config` and ignores any existing `Kodama.toml`. The old `--no-config` flag is retired: it reports an error rather than silently switching to in-place output.

- Recursively discover Markdown and Typst section sources, including `README.md`. Hidden and underscore-prefixed directories remain excluded from source discovery.
- Write `.html` beside each source, overwriting an existing file of the same name. `.typ` files compile to adjacent `.svg` files using the existing cache. No `publish` directory is created.
- Keep caches in `.cache`. Source discovery and watching ignore this directory, and the built-in HTTP server denies access to hidden paths and symlinks escaping the workspace.
- Use ordinary resources in place; no assets directory is required and no assets are copied. Chapter links target relative HTML URLs. Resources in embedded Markdown retain their source-relative meaning.
- Inline the built-in CSS/JS. Do not load workspace `import-*.html` files or the fixed `assets/favicon.ico`. Built-in font and math imports remain available.
- Do not generate RSS. `kodama.json` and `kodama.graph.json` are off by default, but `--indexes` and `--graph` enable them at the workspace root. Disabled outputs are left untouched.
- Do not remove HTML or SVG files when their sources are deleted.
- In `serve`, source and recorded dependency changes trigger compilation. Ordinary resource changes only trigger browser reload. Generated files, temporary files, caches, and ordinary HTML events are ignored. The existing `kodama.reload` file is used for live reload.

This is a local document workflow, not a standalone publish directory: moving only the generated HTML does not move its resources. Visit the desired document URL, such as `/README.html`, when there is no `index.html`.

## `kodama snip`

```sh
kodama snip --section --katex --inline-section
```

Generates VS Code snippet files in `.vscode/`.

Options:

- `--config <path>`, short `-c`: configuration file.
- `--katex`, short `-k`: generate KaTeX snippets.
- `--section`, short `-s`: generate section reference snippets from current section metadata.
- `--inline-section`, short `-i`: generate inline section snippets.

Section snippets refresh in-memory section indexes before writing snippets.

## `kodama upgrade`

```sh
kodama upgrade
kodama upgrade all
kodama upgrade config
kodama upgrade typst-lib
```

Upgrades configuration shape and/or syncs the bundled Typst library into the configured source tree.

Subcommands:

- `all`, alias `a`: upgrade config and sync Typst library. This is the default when no subcommand is supplied.
- `config`, alias `c`: upgrade config only.
- `typst-lib`, alias `t`: sync `kodama.typ` only.

Options for config upgrades:

- `--config <path>`, short `-c`: source configuration file.
- `--output <path>`, short `-o`: write upgraded config to another path instead of overwriting.

Alias: `kodama u`.
