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

// Maintain a rolling timeline of values
const MAX_TIMELINE_LENGTH = 32;
let timelineValues: number[] = [];
let lastUpdateTime = Date.now();
let currentPhase = 0;

function generateNextValue(now: number): number {
  // Update phase based on time passed
  const timeDelta = now - lastUpdateTime;
  currentPhase += (timeDelta / 1000) * Math.PI; // Full cycle every 2 seconds
  lastUpdateTime = now;
  
  // Determine if we're in a silence period (changes every 4 seconds)
  const timeScale = Math.floor(now / 4000);
  const isSilencePeriod = timeScale % 2 === 0;

  if (isSilencePeriod) {
    // During silence, generate very low values
    return Math.random() * 0.08;
  } else {
    // During active periods, generate dynamic values
    const base = 0.6 + 0.3 * Math.sin(currentPhase);
    return Math.max(0, Math.min(1, base + (Math.random() - 0.5) * 0.2));
  }
}

function generateMockData(): WebSocketData {
  const now = Date.now();
  const randomValue = () => Math.floor(Math.random() * 5);
  const generatePosition = () => ({
    x: 10 + Math.random() * 80,
    y: 10 + Math.random() * 80
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

  // Add new value to timeline and maintain max length
  const newValue = generateNextValue(now);
  timelineValues.push(newValue);
  if (timelineValues.length > MAX_TIMELINE_LENGTH) {
    timelineValues = timelineValues.slice(-MAX_TIMELINE_LENGTH);
  }

  return {
    timeline: timelineValues,
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
  // Reset timeline state
  timelineValues = [];
  lastUpdateTime = Date.now();
  currentPhase = 0;
  
  // Mock WebSocket with an EventTarget
  const mockWs = new EventTarget() as WebSocket;
  
  // Send mock data frequently for smooth animation
  const interval = setInterval(() => {
    const mockData = generateMockData();
    callback(mockData);
  }, 50); // Update at 20fps for smooth timeline movement

  // Add cleanup method
  mockWs.close = () => {
    clearInterval(interval);
    // Reset timeline state on close
    timelineValues = [];
    lastUpdateTime = Date.now();
    currentPhase = 0;
  };

  return mockWs;
}