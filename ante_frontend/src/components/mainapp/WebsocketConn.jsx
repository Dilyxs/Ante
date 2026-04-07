import { useEffect, useRef, useState } from "react";
import { WebsocketHandler } from "./WebsocketContext";
const WebsocketConn = ({ children }) => {
  const [incomingMessages, setincomingMessages] = useState([]);
  const [loaded, setloaded] = useState(false);
  const socket = useRef(null);

  useEffect(() => {
    const connect = async () => {
      const baseurl = import.meta.env.VITE_WEBSOCKET_URL;
      try {
        const websocket = new WebSocket(baseurl);
        websocket.onopen = () => {
          setloaded(true);
        };
        websocket.onmessage = (event) => {
          const data = JSON.parse(event.data);
          setincomingMessages((prev) => [...prev, data]);
        };
        websocket.onclose = () => {
          setloaded(false);
        };
        socket.current = websocket;
      } catch (error) {
        console.error("WebSocket connection error:", error);
      }
    };
    connect();
    return () => {
      socket?.current?.close();
    };
  }, []);

  return (
    <div>
      <WebsocketHandler.Provider value={{ loaded, socket, incomingMessages }}>
        {children}
      </WebsocketHandler.Provider>
    </div>
  );
};

export default WebsocketConn;
