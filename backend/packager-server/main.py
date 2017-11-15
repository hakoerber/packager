#!/usr/bin/env python3
from flask import Flask

app = Flask(__name__)

@app.route('/')
def index():
    return "Hello, World!"

@app.route('/api/')
def api():
    return "Hello, API!"

@app.route('/api/v1/')
def apiv1():
    return "Hello, API version 1!"

if __name__ == '__main__':
    app.run(debug=True, port=8000)
