# Helpful Doc References https://github.com/godotengine/godot-docs/tree/master
# https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/gdextension_docs_system.html#publishing-documentation-online

project = "StagToolkit"
copyright = "2026 Alan O'Cull"
author = "Alan O'Cull"
website = "https://alanocull.com/"
version = "0.6.0"

extensions = [
    "sphinx.ext.duration",
    "sphinx.ext.doctest",
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.intersphinx",
    "sphinx_copybutton",
]

html_theme = "furo"
html_title = "StagToolkit"
html_logo = "../../godot/icon.svg"
html_favicon = "https://alanocull.com/favicon.ico"

html_theme_options = {
    "dark_css_variables": {
        "color-brand-primary": "#18d194",
        "color-brand-content": "#18d194",
        # "color-admonition-background": "red",
    },
    "light_css_variables": {
        "color-brand-primary": "#0e5745",
        "color-brand-content": "#0e5745",
        # "color-admonition-background": "red",
    },
}
