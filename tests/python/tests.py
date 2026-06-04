import pytest
import simpleTFTPS
import time
import socket
import os
import pycurl
import hashlib

def test_run():
    def cb_get(file):
        with open("test.txt", "r") as f:
            return f.readall()
        return False
    def cb_put(file):
        return None
    
    # Check it runs and stops
    simpleTFTPS.run("127.0.0.1:9001", cb_get, cb_put)
    time.sleep(0.5)

    buffer = BytesIO()
    c = pycurl.Curl()
    c.setopt(c.URL, 'tftp://127.0.0.1:9001/test')
    c.setopt(c.WRITEDATA, buffer)
    c.setopt(c.CAINFO, certifi.where())
    c.perform()
    c.close()

    body = buffer.getvalue()
    assert hashlib.sha256(body.encode('utf-8')) =="7a32493ca5058aa7065ab15cb6f91b43193109fd87c7d8fdefb26846acf12cc2"
    