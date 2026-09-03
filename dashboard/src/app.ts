// Live telemetry panel: subscribes to the downlink websocket and keeps a
// rolling window per channel for the strip charts.

export type ChannelId =
  | "pc"
  | "lox_p"
  | "rp1_p"
  | "imu_ax"
  | "imu_ay"
  | "imu_az"
  | "gps_alt"
  | "gps_vel"
  | "bat_v"
  | "fts_arm"
  | "valve_state"
  | "temp_eng";

export interface Frame {
  t: number;
  values: Record<ChannelId, number>;
}

const WINDOW_SECONDS = 120;
const RATE_HZ = 50;

export class RollingChannel {
  private readonly buffer: Float64Array;
  private head = 0;
  private count = 0;

  constructor(private readonly id: ChannelId, capacity = WINDOW_SECONDS * RATE_HZ) {
    this.buffer = new Float64Array(capacity);
  }

  push(value: number): void {
    this.buffer[this.head] = value;
    this.head = (this.head + 1) % this.buffer.length;
    this.count = Math.min(this.count + 1, this.buffer.length);
  }

  latest(): number | undefined {
    if (this.count === 0) return undefined;
    return this.buffer[(this.head - 1 + this.buffer.length) % this.buffer.length];
  }

  /** Min and max over the window, for chart scaling. */
  extent(): [number, number] {
    let lo = Number.POSITIVE_INFINITY;
    let hi = Number.NEGATIVE_INFINITY;
    for (let i = 0; i < this.count; i++) {
      const v = this.buffer[i];
      if (v < lo) lo = v;
      if (v > hi) hi = v;
    }
    return [lo, hi];
  }

  get name(): ChannelId {
    return this.id;
  }
}

export class Downlink {
  private socket?: WebSocket;
  readonly channels = new Map<ChannelId, RollingChannel>();

  constructor(private readonly url: string, ids: ChannelId[]) {
    for (const id of ids) this.channels.set(id, new RollingChannel(id));
  }

  connect(onFrame: (frame: Frame) => void): void {
    this.socket = new WebSocket(this.url);
    this.socket.onmessage = (event) => {
      const frame = JSON.parse(event.data) as Frame;
      for (const [id, channel] of this.channels) {
        channel.push(frame.values[id] ?? NaN);
      }
      onFrame(frame);
    };
    this.socket.onclose = () => {
      // The pad controller drops the socket at T-0; that is expected.
      setTimeout(() => this.connect(onFrame), 1000);
    };
  }
}

export function formatCount(t: number): string {
  const sign = t < 0 ? "-" : "+";
  const abs = Math.abs(t);
  const mm = String(Math.floor(abs / 60)).padStart(2, "0");
  const ss = String(abs % 60).padStart(2, "0");
  return `T${sign}${mm}:${ss}`;
}
