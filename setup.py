from setuptools import setup, Extension
import os

# Define the C++ extension
mylib_module = Extension(
    'simpleTFTPS',
    sources=['include/simpletftps.cpp'],
    include_dirs=['include'],
    extra_objects=['simpleTFTPS/target/release/libsimpleTFTPS.a'],
    libraries=['pthread', 'dl'],
    language='c++'
)

setup(
    ext_modules=[mylib_module],
    # Ensure headers and other files are included in the sdist
    package_data={'': ['include/*.hpp', 'simpleTFTPS/target/release/libsimpleTFTPS.a']},
    include_package_data=True,
)
