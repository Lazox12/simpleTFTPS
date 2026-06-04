import pytest
import simpleTFTPS

def test_get():
    def callback_get(file):
        data:str
        with open(file,"r") as f:
            data = f.readall()
    return data

    def callback_put(file):
        return False

    run_py("127.0.0.1:9001",callback_get,callback_put)