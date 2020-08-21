from contextlib import suppress

import css_inline
from hypothesis import strategies as st, given

def test_simple():
    html = """
<html>
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
</html>"""
    expected = """<html><head>
    
</head>
<body>
    <h1 style=" color:blue; ">Big Text</h1>

</body></html>"""
    assert css_inline.inline(html, True) == expected


@given(html=st.text(), remove_style_tags=st.booleans())
def test_smoke(html, remove_style_tags):
    with suppress(css_inline.InlineError):
        css_inline.inline(html, remove_style_tags)
