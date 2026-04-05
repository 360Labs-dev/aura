/**
 * Aura Language — npm package
 *
 * This package provides:
 * - TextMate grammar for syntax highlighting (editors/IDEs)
 * - Example .aura programs
 * - Language reference
 *
 * The Aura compiler is written in Rust. Install it with:
 *   git clone https://github.com/360Labs-dev/aura.git
 *   cd aura && cargo install --path crates/aura-cli
 *
 * Or build from source:
 *   cargo build --release
 *   ./target/release/aura build --target web
 */

const path = require('path');

module.exports = {
  name: 'aura-lang',
  version: '0.1.1',

  /**
   * Path to the TextMate grammar file for Aura syntax highlighting.
   * Use this to integrate Aura highlighting into any editor that supports TextMate grammars.
   */
  grammarPath: path.join(__dirname, 'syntaxes', 'aura.tmLanguage.json'),

  /**
   * Path to example .aura files.
   */
  examplesPath: path.join(__dirname, 'examples'),

  /**
   * Aura file extension.
   */
  fileExtension: '.aura',

  /**
   * Language ID for editor integration.
   */
  languageId: 'aura',

  /**
   * Aura language keywords.
   */
  keywords: [
    'app', 'screen', 'view', 'model', 'state', 'action', 'each', 'if', 'else',
    'when', 'is', 'import', 'from', 'as', 'theme', 'style', 'true', 'false',
    'nil', 'and', 'or', 'not', 'enum', 'fn', 'return', 'let', 'const',
    'list', 'map', 'set', 'optional', 'component', 'navigate', 'emit', 'on',
    'animate', 'with', 'where', 'slot',
  ],

  /**
   * Aura built-in types.
   */
  types: [
    'text', 'int', 'float', 'bool', 'timestamp', 'duration', 'percent',
    'secret', 'sanitized', 'email', 'url', 'token',
  ],

  /**
   * Aura design token categories.
   */
  designTokens: {
    spacing: ['xs', 'sm', 'md', 'lg', 'xl', '2xl', '3xl', '4xl'],
    color: ['primary', 'secondary', 'muted', 'accent', 'danger', 'warning', 'success', 'info', 'surface', 'background'],
    typography: ['bold', 'medium', 'semibold', 'italic', 'mono', 'underline', 'strike', 'center', 'uppercase'],
    shape: ['sharp', 'subtle', 'rounded', 'smooth', 'pill', 'circle'],
    motion: ['ease', 'spring', 'bounce', 'instant', 'fast', 'normal', 'slow'],
  },

  /**
   * View element keywords.
   */
  viewElements: {
    layout: ['column', 'row', 'stack', 'grid', 'scroll', 'wrap'],
    widgets: ['text', 'heading', 'image', 'icon', 'badge', 'divider', 'spacer', 'progress', 'avatar'],
    inputs: ['button', 'textfield', 'textarea', 'checkbox', 'toggle', 'slider', 'picker', 'datepicker', 'segmented', 'stepper'],
  },
};
