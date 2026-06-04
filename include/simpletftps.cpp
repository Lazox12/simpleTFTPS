
#include "simpletftps.hpp"
#include <csignal>
#include <python3.14/Python.h>
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
        PyErr_SetString(PyExc_TypeError, "Parameter must be callable!");
        return nullptr;
    }
    if (!PyCallable_Check(tmp_callback_put)) {
        PyErr_SetString(PyExc_TypeError, "Parameter must be callable!");
        return nullptr;
    }

    Py_INCREF(tmp_callback_get);
    Py_INCREF(tmp_callback_put);
    Py_DECREF(callback_get);
    Py_DECREF(callback_put);
    callback_get = tmp_callback_get;
    callback_put = tmp_callback_put;

    pid_t pid = fork();
    if (pid == 0) {
        run(inner_get, inner_put, address);
    }
    else instance = pid;

}
char* inner_get(const char* file) {
    if (callback_get == nullptr) {
        printf("No callback registered!\n");
        return nullptr;
    }
    PyObject *arglist = Py_BuildValue("(s)", file);
    PyObject *result = PyObject_CallObject(callback_get, arglist);
    Py_DECREF(arglist);
    if (result == nullptr) {
        return nullptr;
    }
    const char* py_ret_str = PyUnicode_AsUTF8(result);
    char* c_ret_str = strdup(py_ret_str);
    Py_DECREF(result);
    return c_ret_str;
}
char* inner_put(const char* file) {
    if (callback_put == nullptr) {
        printf("No callback registered!\n");
        return nullptr;
    }
    PyObject *arglist = Py_BuildValue("(s)", file);
    PyObject *result = PyObject_CallObject(callback_put, arglist);
    Py_DECREF(arglist);
    if (result == nullptr) {
        return nullptr;
    }
    const char* py_ret_str = PyUnicode_AsUTF8(result);
    char* c_ret_str = strdup(py_ret_str);
    Py_DECREF(result);
    return c_ret_str;
}

PyObject* py_stop(PyObject* self, PyObject* args) {
    if (instance !=0) {
        PyErr_SetString(PyExc_RuntimeError, "Server not running!\n");
        return nullptr;
    }
    kill(instance,SIGTERM);

    Py_RETURN_TRUE;
}