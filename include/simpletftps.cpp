
#include <Python.h>
#include "simpletftps.hpp"
#include <csignal>
#include <unistd.h>
#include <cstring>
#include <cstdio>
#include <sys/types.h>

static PyObject* callback_get = nullptr;
static PyObject* callback_put = nullptr;

char* inner_get(const char* file);
char* inner_put(const char* file);

static PyObject* py_run(PyObject* self, PyObject* args) {
    char* address;
    PyObject* tmp_callback_get;
    PyObject* tmp_callback_put;

    if (!PyArg_ParseTuple(args, "sOO", &address, &tmp_callback_get, &tmp_callback_put)) {
        return nullptr;
    }
    if (!PyCallable_Check(tmp_callback_get)) {
        PyErr_SetString(PyExc_TypeError, "Parameter 2 must be callable!");
        return nullptr;
    }
    if (!PyCallable_Check(tmp_callback_put)) {
        PyErr_SetString(PyExc_TypeError, "Parameter 3 must be callable!");
        return nullptr;
    }

    Py_XINCREF(tmp_callback_get);
    Py_XINCREF(tmp_callback_put);
    // Note: in a more robust implementation, we'd handle thread safety for these globals
    callback_get = tmp_callback_get;
    callback_put = tmp_callback_put;

    Py_BEGIN_ALLOW_THREADS
    run(inner_get, inner_put, address);
    Py_END_ALLOW_THREADS

    Py_XDECREF(tmp_callback_get);
    Py_XDECREF(tmp_callback_put);

    Py_RETURN_NONE;
}

char* inner_get(const char* file) {
    PyGILState_STATE gstate = PyGILState_Ensure();
    
    if (callback_get == nullptr) {
        PyGILState_Release(gstate);
        return nullptr;
    }
    
    PyObject *arglist = Py_BuildValue("(s)", file);
    PyObject *result = PyObject_CallObject(callback_get, arglist);
    Py_DECREF(arglist);
    
    if (result == nullptr) {
        if (PyErr_Occurred()) PyErr_Print();
        PyGILState_Release(gstate);
        return nullptr;
    }
    if (result == Py_None) {
        Py_DECREF(result);
        PyGILState_Release(gstate);
        return nullptr;
    }
    
    char* c_ret_str = nullptr;
    if (PyUnicode_Check(result)) {
        const char* py_ret_str = PyUnicode_AsUTF8(result);
        if (py_ret_str) {
            c_ret_str = strdup(py_ret_str);
        }
    } else if (PyBytes_Check(result)) {
        const char* py_ret_str = PyBytes_AsString(result);
        if (py_ret_str) {
            c_ret_str = strdup(py_ret_str);
        }
    }
    
    Py_DECREF(result);
    PyGILState_Release(gstate);
    return c_ret_str;
}

char* inner_put(const char* file) {
    PyGILState_STATE gstate = PyGILState_Ensure();

    if (callback_put == nullptr) {
        PyGILState_Release(gstate);
        return nullptr;
    }
    
    PyObject *arglist = Py_BuildValue("(s)", file);
    PyObject *result = PyObject_CallObject(callback_put, arglist);
    Py_DECREF(arglist);
    
    if (result == nullptr) {
        if (PyErr_Occurred()) PyErr_Print();
        PyGILState_Release(gstate);
        return nullptr;
    }
    if (result == Py_None) {
        Py_DECREF(result);
        PyGILState_Release(gstate);
        return nullptr;
    }
    
    char* c_ret_str = nullptr;
    if (PyUnicode_Check(result)) {
        const char* py_ret_str = PyUnicode_AsUTF8(result);
        if (py_ret_str) {
            c_ret_str = strdup(py_ret_str);
        }
    } else if (PyBytes_Check(result)) {
        const char* py_ret_str = PyBytes_AsString(result);
        if (py_ret_str) {
            c_ret_str = strdup(py_ret_str);
        }
    }
    
    Py_DECREF(result);
    PyGILState_Release(gstate);
    return c_ret_str;
}

static PyObject* py_stop(PyObject* self, PyObject* args) {
    // Stop is no longer supported this way as we are synchronous
    // and don't manage a separate process anymore.
    Py_RETURN_NONE;
}

static PyMethodDef SimpleTFTPSMethods[] = {
    {"run", py_run, METH_VARARGS, "Run the TFTPS server (blocking)"},
    {"stop", py_stop, METH_VARARGS, "Stop the TFTPS server (no-op)"},
    {NULL, NULL, 0, NULL}
};

static struct PyModuleDef simpletftpsmodule = {
    PyModuleDef_HEAD_INIT,
    "simpleTFTPS",
    NULL,
    -1,
    SimpleTFTPSMethods
};

PyMODINIT_FUNC PyInit_simpleTFTPS(void) {
    return PyModule_Create(&simpletftpsmodule);
}
