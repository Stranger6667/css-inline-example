from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="css-inline",
    version="0.1.0",
    rust_extensions=[RustExtension("css_inline", binding=Binding.PyO3, strip=1)],
    zip_safe=False,
)
