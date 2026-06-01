<div align="center">
  <h1>Alloy OS</h1>
  <p><strong>An operating system built in Rust, C/C++, and Assembly.</strong></p>
  <img src="assets/alloy-os-light.svg" alt="Alloy OS light logo" width="300" />
  <img src="assets/alloy-os-dark.svg" alt="Alloy OS dark logo" width="300" />
</div>

<p align="center">
  <em>Kernel boot, Fusion display, Wayland support, and desktop runtime all live in this repo.</em>
</p>

<h2>Current state</h2>

<p align="center">
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/codeql-advanced.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/codeql-advanced.yml/badge.svg" alt="Advanced CodeQL Analysis"></a>
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/build-and-test.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/build-and-test.yml/badge.svg" alt="Build and Test Kernel"></a>
  <a href="https://github.com/fkm-X3/Alloy-OS/actions/workflows/rust-clippy.yml"><img src="https://github.com/fkm-X3/Alloy-OS/actions/workflows/rust-clippy.yml/badge.svg" alt="rust-clippy analyze"></a>
</p>

<p>
Alloy OS boots the Rust display server in the Iced-primary software-rendered mode. Fusion is the compositor/backend layer.
</p>

<h2>Build and run</h2>

<p>The repo works natively on Linux, but Docker is the recommended path for a reproducible setup that matches CI.</p>

<h3>Docker workflow</h3>

<p>Docker is the easiest reproducible path and keeps local builds close to CI.</p>

<pre><code>docker compose build
docker compose run --rm alloy make output
docker compose run --rm alloy make screenshot
docker compose run --rm alloy make mouse-smoke
docker compose run --rm alloy make mouse-screenshot</code></pre>

<p>The repo is bind-mounted into <code>/workspace</code>, so edits on the host are visible inside the container immediately.</p>

<p>Or run it directly:</p>

<pre><code>docker build -t alloy-os-dev .
docker run --rm -it -v "$PWD:/workspace" -w /workspace alloy-os-dev make output
make docker-run</code></pre>

<p><code>make run</code> opens a GUI QEMU window and may need display forwarding in Docker. The headless targets (<code>make output</code>, <code>make screenshot</code>, <code>make mouse-smoke</code>, <code>make mouse-screenshot</code>) work well in containers.</p>

<pre><code>make iso
make lazy
make run
make output
make screenshot
make mouse-smoke
make mouse-screenshot
make clean</code></pre>

<details>
  <summary>What each target does</summary>

  <ul>
    <li><code>make iso</code> builds the kernel and produces a bootable image.</li>
    <li><code>make lazy</code> does a clean rebuild and then produces the ISO.</li>
    <li><code>make run</code> boots QEMU with a visible display.</li>
    <li><code>make output</code> boots headless and prints serial output only.</li>
    <li><code>make screenshot</code> boots headless and captures the first Cosmos desktop frame.</li>
    <li><code>make mouse-smoke</code> runs scripted mouse interactions against the boot image.</li>
    <li><code>make mouse-screenshot</code> runs the mouse flow and saves a proof screenshot.</li>
    <li><code>make clean</code> removes build outputs.</li>
  </ul>
</details>

<h2>What you see at boot (currently screenshot updater is broken)</h2>

<p align="center">
  <img src="assets/desktop-shell-grid.png" alt="Alloy OS desktop shell screenshot" />
</p>

<ul>
  <li>The kernel enters Rust through <code>kernel/rust/src/lib.rs::rust_main()</code>.</li>
  <li>VFS is initialized early.</li>
  <li>The display server boots from <code>kernel/rust/src/display_server.rs</code>.</li>
  <li>Fusion composites Cosmos DE shell surfaces by default; Iced-primary remains available as a non-default mode.</li>
  <li>Wayland support lives under <code>kernel/rust/src/fusion/wayland</code>.</li>
</ul>

<h2>Controls</h2>

<ul>
  <li><strong>ESC</strong>: exit display mode</li>
  <li><strong>`</strong>: toggle keyboard window-control mode</li>
  <li><strong>1 / 2 / 3 / 4</strong>: switch Iced tabs in normal mode</li>
  <li><strong>P</strong>: toggle the palette overlay</li>
  <li><strong>T / Space</strong>: toggle accent brightness</li>
  <li><strong>PgUp / PgDn</strong>: cycle focused window</li>
  <li><strong>Arrow keys</strong>: move the focused window in control mode</li>
  <li><strong>+ / -</strong>: resize the focused window in control mode</li>
  <li><strong>M</strong>: minimize the focused window in control mode</li>
  <li><strong>H</strong>: hide the focused window in control mode</li>
  <li><strong>R</strong>: restore the next hidden or minimized window</li>
  <li><strong>C / X</strong>: close the focused window in control mode</li>
  <li><strong>Mouse move</strong>: moves the on-screen pointer</li>
  <li><strong>Left click</strong>: focuses the top-most window under the pointer</li>
  <li><strong>Left-drag</strong> on the title bar: drags the focused window</li>
</ul>

<p>
Mouse input is relative PS/2 input, so click inside the QEMU window to grab pointer focus. <code>make output</code> is headless and cannot capture live host mouse input; use <code>make run</code> or <code>make mouse-smoke</code> for pointer testing.
</p>

<h2>Terminal mode</h2>

<p>If Alloy falls back to terminal mode, these built-ins are available:</p>

<ul>
  <li><code>help [command]</code> - list commands or show detailed help</li>
  <li><code>clear</code>, <code>echo</code>, <code>version</code></li>
  <li><code>sysinfo</code>, <code>uname</code>, <code>free</code>, <code>ticks</code></li>
  <li><code>meminfo</code>, <code>cpuinfo</code>, <code>uptime</code></li>
</ul>

<h2>Repository layout</h2>

<ul>
  <li><code>kernel/c</code> - early boot, drivers, paging, and handoff code</li>
  <li><code>kernel/rust</code> - Rust kernel entry, Fusion integration, and display-server runtime</li>
  <li><code>os/display</code> - shared display server library, apps, protocol, and backend abstractions</li>

  <li><code>boot</code> - bootloader and assembly entry pieces</li>
  <li><code>tools</code> - screenshot and smoke-test helpers</li>
</ul>

<h2>Validation</h2>

<ul>
  <li><code>cd os/display &amp;&amp; cargo test --manifest-path Cargo.toml</code></li>
  <li><code>cd os/display &amp;&amp; cargo test --manifest-path Cargo.toml &lt;test_name&gt;</code></li>
  <li><code>cd kernel/rust &amp;&amp; cargo +nightly fmt --check</code></li>
  <li><code>cd kernel/rust &amp;&amp; cargo +nightly clippy --target i686-alloy.json -Zbuild-std=core,alloc</code></li>
  <li><code>nasm -f elf32 -o /dev/null &lt;file.asm&gt;</code></li>
</ul>

<h2>Notes</h2>

<ul>
  <li>The display-server + window-manager path is still gated behind the existing boot flow.</li>

  <li>The repo currently builds around the kernel + shared display library split instead of a Cargo workspace root.</li>
</ul>
