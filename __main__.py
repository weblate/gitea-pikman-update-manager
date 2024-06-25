#! /bin/python3

import socket
import os

import sys
import time

import apt_pkg

import apt
import apt.progress.base

def get_change(current, total):
    if current == total:
        return 100.0
    try:
        return float("{:.1f}".format(((current * 100) / total))) 
    except ZeroDivisionError:
        return 0.0

class UpdateProgressSocket(apt.progress.base.AcquireProgress):
    # Init
    def __init__(self):
        pass

    # Start
    def start(self):
        self.current_bytes = 0
        self.total_bytes = 0
        print("Starting APT Cache Update.")
        return super().start()
    
    # Stop
    def stop(self):
        print("\nAPT Cache Update Complete!")
        return super().stop()

    # Progrss pulse
    def pulse(self, owner):
        # Calculate current progress percentage
        progress_percent = get_change(self.current_bytes, self.total_bytes)
        
        # apt_update_progress ipc sock
        socket_path = "/tmp/pika_apt_update.sock"
        
        # Create a Unix domain socket
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(socket_path)
            # Send percentage to socket as UTF-8
            client.sendall(str(progress_percent).encode('utf-8'))
        
        #response = client.recv(1024)
        #print(f"Received: {response.decode('utf-8')}")
        
        return True

    
    def fail(self, item):
        print("Failure at: %s %s" % (item.uri, item.shortdesc))

    def fetch(self, item):
        print("Fetch: %s %s" % (item.uri, item.shortdesc))

    def ims_hit(self, item):
        print("Download: %s %s" % (item.uri, item.shortdesc))

    def media_change(self, medium, drive):
        print(f"Please insert medium {medium} in drive {drive}")
        sys.stdin.readline()
        # return False

def update_cache():
    # First of all, open the cache
    cache = apt.Cache()
    # Now, lets update the package list
    cache.update(UpdateProgressSocket())
    # We need to re-open the cache because it needs to read the package list
    cache.open(None)
    # We need to re-open the cache because it needs to read the package list
    for pkg in cache:
        if pkg.is_upgradable:
            print(f"{pkg.name} ({pkg.installed.version} -> {pkg.candidate.version})")

def process(data):
    # Echo the input data
    return data

if __name__ == "__main__":

    update_cache()
