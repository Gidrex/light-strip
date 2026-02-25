import requests
import numpy as np
import threading
import time
from zeroconf import ServiceBrowser, Zeroconf
import socket

class feelinlight:
    def __init__(self, name):
        self.name = name
        self.ip_list = []
        self.thread_list = []
        self.headers = {
            'Content-Type': 'application/x-www-form-urlencoded'
        }

    def find_devices(self, delay):
        class MyListener:
            def __init__(self, parent):
                self.parent = parent

            def remove_service(self, zeroconf, type, name):
                pass

            def add_service(self, zeroconf, type, name):
                info = zeroconf.get_service_info(type, name)
                if info and "FeelinLight" in info.name:
                    # Zeroconf returns addresses as bytes
                    for addr in info.addresses:
                        ip = socket.inet_ntoa(addr)
                        if ip not in self.parent.ip_list:
                            self.parent.ip_list.append(ip)
                            print(f"Device Name: {info.name.split('.')[0]} Device IP address: {ip}")

            def update_service(self, zeroconf, type, name):
                pass

        zeroconf = Zeroconf()
        listener = MyListener(self)
        browser = ServiceBrowser(zeroconf, '_http._tcp.local.', listener)
        time.sleep(delay)
        zeroconf.close()

    def send_post(self, data):
        self.thread_list = []
        if len(self.ip_list) == 0:
            print("No devices found. Please run find_devices() first or set ip_list manually.")
            return

        if len(self.ip_list) == 1:
            url = f"http://{self.ip_list[0]}/echo"
            self.post(url, data)
        else:
            for ip in self.ip_list:
                url = f"http://{ip}/echo"
                t = threading.Thread(target=self.post, args=(url, data))
                self.thread_list.append(t)
                print(f'Adding thread for {url}***')
            
            for t in self.thread_list:
                t.start()
                print('Starting thread***')
            
            # Wait for threads to finish (optional, but keep consistent with original)
            for t in self.thread_list:
                t.join(timeout=1.0)

    def _build_packet(self, prefix, cmd, *args):
        # The protocol uses a checksum which is a simple sum of all bytes
        payload = list(prefix) + [cmd] + list(args)
        sum_mac = sum(payload)
        # Final packet is payload + (sum & 0xFF)
        packet = bytearray(payload + [sum_mac & 0xFF])
        return packet

    def whole_lamp_color(self, R, G, B):
        m = self._build_packet([82, 66, 9, 1], 7, R, G, B)
        self.send_post(m)

    def single_lamp(self, number, R, G, B):
        m = self._build_packet([82, 66, 10, 1], 2, number, R, G, B)
        self.send_post(m)

    def beep(self, state):
        if state == 'on':
            m = bytearray([82, 66, 7, 1, 4, 3, 163])
            print('BEEP is open')
            self.send_post(m)
        elif state == 'off':
            m = bytearray([82, 66, 7, 1, 4, 4, 164])
            print('BEEP is close')
            self.send_post(m)

    def brightness(self, number):
        m = self._build_packet([82, 66, 7, 1], 6, number)
        self.send_post(m)

    def basic_fantasy(self, number):
        if number < 6:
            num = number + 32
            m = self._build_packet([82, 66, 7, 1], 3, num)
        else:
            num = number - 6
            m = self._build_packet([82, 66, 7, 1], 33, num)
        self.send_post(m)

    def body_sensor(self, val):
        m = self._build_packet([82, 66, 7, 1], 10, val)
        self.send_post(m)
        return m

    def basic_sound_pickup(self, number):
        if 0 <= number < 6:
            num = number + 48
            m = self._build_packet([82, 66, 7, 1], 3, num)
            self.send_post(m)

    def equipment_switch(self, number):
        if 0 < number <= 2:
            m = self._build_packet([82, 66, 7, 1], 4, number)
            self.send_post(m)

    def mode(self, val):
        m = self._build_packet([82, 66, 7, 1], 13, val)
        self.send_post(m)

    def single_set(self, index, R, G, B):
        m = self._build_packet([82, 66, 10, 1], 8, index, R, G, B)
        self.send_post(m)

    def edit_fantasy(self, number):
        if 0 <= int(number) < 5:
            num = int(number) + 113
            m = self._build_packet([82, 66, 7, 1], 3, num)
            self.send_post(m)

    def edit_fantasy_speed(self, val):
        m = self._build_packet([82, 66, 7, 1], 11, val)
        self.send_post(m)
        return m

    def edit_sound_pickup(self, number):
        if 0 <= int(number) < 5:
            num = int(number) + 161
            m = self._build_packet([82, 66, 7, 1], 3, num)
            self.send_post(m)

    def animation_monochrome(self, index, *arrs):
        combined_arr = []
        for a in arrs:
            if isinstance(a, (list, tuple)):
                combined_arr.extend(a)
            else:
                combined_arr.append(a)
        
        send_str = [82, 66, 55, 1, 15, index] + combined_arr
        sum_mac = sum(send_str)
        m = bytearray(send_str + [sum_mac & 0xFF])
        self.send_post(m)

    def animation_speed(self, number):
        if 50 <= int(number) <= 500:
            jiange = int(number / 10)
            m = self._build_packet([82, 66, 7, 1], 32, jiange)
            self.send_post(m)

    def firmware_update(self):
        m = bytearray([82, 66, 7, 1, 12, 0, 168])
        self.send_post(m)

    def post(self, url, data):
        try:
            requests.post(url, headers=self.headers, data=data, timeout=5)
        except Exception as e:
            print(f"Error sending POST to {url}: {e}")
