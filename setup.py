from setuptools import setup, Extension

# Define the C extension
mylib_module = Extension(
    'simpleTFTPS-c',               # The name of the module you will import
    sources=['include/simpletftps.c','include/simpletftps.h']    # The C source files to compile
)

setup(
    name='simpleTFTPS',
    version='0.1.0',
    description='a simple TFTPS client',
    ext_modules=[mylib_module], # Tell setuptools to build this extension
)