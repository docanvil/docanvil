# Bundled Devicon icons

Source: https://github.com/devicons/devicon/tree/v2.17.0/icons
Release: v2.17.0. Each upstream SVG is the unmodified `<name>-original.svg` asset.
`bash-light.svg` is a local variant of `bash.svg`: the dark `#293138` fill
is replaced with `#a8b9cc` for contrast on code headers; the green detail is retained.
Licence: MIT; see LICENSE.txt (included in generated sites).

The `codeLanguageIcons` map in ../../docanvil.js selects the SVG, accent colour,
monochrome treatment, and fence-language aliases. Original multicolour logos
keep their own fills; monochrome SVGs are tinted with the mapped colour using
CSS masks. To add a language, vendor its SVG from the pinned release and add
an entry to the map. No CDN or runtime package is required.
