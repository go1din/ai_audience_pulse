interface WebSocketData {
  timeline: number[];
  emojis: {
    thumbs: number;
    applause: number;
    smile: number;
  };
  applauseIntensity: number;
  reactionEvents?: {
    type: 'thumbs' | 'applause' | 'smile';
    position: { x: number; y: number };
    timestamp: number;
  }[];
}

type WebSocketCallback = (data: WebSocketData) => void;

function generateMockData(): WebSocketData {
  const now = Date.now();
  const randomValue = () => Math.floor(Math.random() * 5);
  const generatePosition = () => ({
    x: 10 + Math.random() * 80, // 10-90% of screen width
    y: 10 + Math.random() * 80  // 10-90% of screen height
  });

  // Generate random reaction events
  const reactionEvents = [];
  if (Math.random() > 0.5) {
    const types = ['thumbs', 'applause', 'smile'] as const;
    const randomType = types[Math.floor(Math.random() * types.length)];
    reactionEvents.push({
      type: randomType,
      position: generatePosition(),
      timestamp: now
    });
  }

  return {
    timeline: Array(10).fill(0).map(() => Math.random() * 100),
    emojis: {
      thumbs: randomValue(),
      applause: randomValue(),
      smile: randomValue()
    },
    applauseIntensity: Math.random(),
    reactionEvents
  };
}

export function connectWebSocket(callback: WebSocketCallback): WebSocket {
  // Mock WebSocket with an EventTarget
  const mockWs = new EventTarget() as WebSocket;
  
  // Send mock data periodically
  const interval = setInterval(() => {
    const mockData = generateMockData();
    callback(mockData);
  }, 2000);

  // Add cleanup method
  mockWs.close = () => {
    clearInterval(interval);
  };

  return mockWs;
}