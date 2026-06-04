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
    name='simpleTFTPS',
    description='a simple TFTPS client',
    ext_modules=[mylib_module],
    packages=[],
    package_data={'': ['include/*.hpp', 'simpleTFTPS/target/release/libsimpleTFTPS.a']},
    include_package_data=True,
    use_scm_version=True,
    setup_requires=['setuptools_scm'],
)
