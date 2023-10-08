import socket

def main():
    # 1.创建一个udp套接字
    udp_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    # 2.准备接收方的地址
    for i in range(5):
        udp_socket.setblocking(False)
        msg = "hello" + str(i)
        udp_socket.sendto(msg.encode("utf-8"), ("172.16.1.2", 80))
    
    udp_socket.close()

main()