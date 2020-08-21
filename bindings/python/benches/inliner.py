import inlinestyler.utils
import premailer
import pynliner
import pytest

import css_inline

SIMPLE_HTML = """<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1>Big Text</h1>
    <p>
        <strong>Solid</strong>
    </p>
    <p class="footer">Foot notes</p>
</body>
</html>"""


@pytest.mark.parametrize(
    "func",
    (
        css_inline.inline,
        premailer.transform,
        pynliner.fromString,
        inlinestyler.utils.inline_css,
    ),
    ids=("css_inline", "premailer", "pynliner", "inlinestyler"),
)
@pytest.mark.benchmark(group="simple")
def test_simple(benchmark, func):
    benchmark(func, SIMPLE_HTML)
