import socket
import time

def main():
    tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    
    server_addr = ("172.16.1.2", 80)
    tcp_socket.connect(server_addr)
    tcp_socket.settimeout(100)
    time.sleep(2)
    send_data = "connect ok?"
    tcp_socket.send(send_data.encode("utf8"))
    # tcp_socket.send(send_data.encode("utf8"))
    # tcp_socket.send(send_data.encode("utf8"))
    # recv_data = tcp_socket.recv(1024)
    # print(recv_data)
    # recv_data = tcp_socket.recv(1024)
    # print(recv_data)
    tcp_socket.close()

main()
