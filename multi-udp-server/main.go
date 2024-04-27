package main

import (
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"
)

func main() {
	var portsStr string
	flag.StringVar(&portsStr, "ports", "", "Comma-separated list of port numbers to listen on")
	flag.Parse()

	if portsStr == "" {
		log.Fatal("No ports specified. Usage example: --ports=9876,9877,9878")
	}

	ports, err := parsePorts(portsStr)
	if err != nil {
		log.Fatalf("Error parsing ports: %v", err)
	}

	// Setup logger
	logger := log.New(os.Stdout, "UDP Server: ", log.LstdFlags)

	// Channel to listen for errors or successful listener setup
	listeners := make([]net.PacketConn, len(ports))
	errs := make(chan error)
	quit := make(chan os.Signal, 1)

	// Signal handling for graceful shutdown
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	// Start all servers
	for i, port := range ports {
		conn, err := runServer(port, logger)
		if err != nil {
			logger.Printf("Failed to start server on port %d: %v", port, err)
			continue
		}
		listeners[i] = conn
		go handlePackets(conn, port, logger, errs)
	}

	// Wait for signal or error
	select {
	case <-quit:
		logger.Println("Shutting down servers...")
	case err := <-errs:
		logger.Printf("Error: %v", err)
	}

	// Close all listeners
	for _, listener := range listeners {
		if listener != nil {
			listener.Close()
		}
	}
}

func parsePorts(portsStr string) ([]int, error) {
	var ports []int
	for _, p := range strings.Split(portsStr, ",") {
		port, err := strconv.Atoi(p)
		if err != nil {
			return nil, fmt.Errorf("invalid port '%s'", p)
		}
		ports = append(ports, port)
	}
	return ports, nil
}

func runServer(port int, logger *log.Logger) (net.PacketConn, error) {
	addr := fmt.Sprintf("127.0.0.1:%d", port)
	conn, err := net.ListenPacket("udp", addr)
	if err != nil {
		return nil, err
	}
	logger.Printf("Listening on %s", addr)
	return conn, nil
}

func handlePackets(conn net.PacketConn, port int, logger *log.Logger, errs chan<- error) {
	buf := make([]byte, 1024)
	for {
		n, remoteAddr, err := conn.ReadFrom(buf)
		if err != nil {
			errs <- fmt.Errorf("read error on port %d: %w", port, err)
			return
		}

		logger.Printf("Port %d: %d bytes received from %s", port, n, remoteAddr)
		logger.Printf("Port %d: buffer contents: %s", port, string(buf[:n]))
	}
}
