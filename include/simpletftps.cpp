
#include "simpletftps.hpp"
#include <csignal>
#include <python3.14/Python.h>
#include <unistd.h>
#include <cstring>
#include <cstdio>
#include <sys/types.h>

static PyObject* callback_get = nullptr;
static PyObject* callback_put = nullptr;
pid_t instance = 0;

char* inner_get(const char* file);
char* inner_put(const char* file);

static PyObject* py_run(PyObject* self, PyObject* args) {
    if (instance != 0) {
        PyErr_SetString(PyExc_RuntimeError, "Server already running!");
        return nullptr;
    }
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
    Py_XDECREF(callback_get);
    Py_XDECREF(callback_put);
    callback_get = tmp_callback_get;
    callback_put = tmp_callback_put;

    // Use PyOS_BeforeFork if available, but for now let's just fork
    pid_t pid = fork();
    if (pid == 0) {
        // Child process
        PyOS_AfterFork_Child();
        
        // In the child, we are the only thread.
        // The GIL is held by us.
        // We should release it before calling the blocking run()
        // but since run() spawns threads that call back into Python,
        // we need to make sure the GIL is available for them.
        
        Py_BEGIN_ALLOW_THREADS
        run(inner_get, inner_put, address);
        Py_END_ALLOW_THREADS
        _exit(0); 
    }
    else if (pid < 0) {
        PyErr_SetFromErrno(PyExc_OSError);
        return nullptr;
    }
    else {
        instance = pid;
        PyOS_AfterFork_Parent();
    }
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
    const char* py_ret_str = PyUnicode_AsUTF8(result);
    char* c_ret_str = nullptr;
    if (py_ret_str) {
        c_ret_str = strdup(py_ret_str);
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
    const char* py_ret_str = PyUnicode_AsUTF8(result);
    char* c_ret_str = nullptr;
    if (py_ret_str) {
        c_ret_str = strdup(py_ret_str);
    }
    Py_DECREF(result);
    
    PyGILState_Release(gstate);
    return c_ret_str;
}

static PyObject* py_stop(PyObject* self, PyObject* args) {
    if (instance == 0) {
        PyErr_SetString(PyExc_RuntimeError, "Server not running!");
        return nullptr;
    }
    kill(instance, SIGTERM);
    instance = 0;

    Py_RETURN_TRUE;
}

static PyMethodDef SimpleTFTPSMethods[] = {
    {"run", py_run, METH_VARARGS, "Run the TFTPS server"},
    {"stop", py_stop, METH_VARARGS, "Stop the TFTPS server"},
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
