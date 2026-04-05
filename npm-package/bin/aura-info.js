#!/usr/bin/env node

console.log(`
  Aura Language v0.1.0
  The programming language for AI coding agents.

  Install the compiler:
    git clone https://github.com/360Labs-dev/aura.git
    cd aura && cargo build --release

  Quick start:
    aura sketch "todo app with dark mode"
    aura build sketch.aura --target web
    aura build sketch.aura --target all
    aura init myapp && cd myapp && aura run

  Commands:
    aura build [path] --target <web|ios|android|all>
    aura run                          Dev server
    aura sketch "<description>"       Generate from English
    aura init <name>                  New project
    aura fmt [path]                   Format code
    aura explain <file>               Code to English
    aura diff <a> <b>                 Semantic diff
    aura doctor                       Check environment
    aura agent serve                  AI Agent API

  Docs: https://github.com/360Labs-dev/aura
`);
