export class Entropy {
  constructor(bufferSize = 1024) {
    this.bufferSize = bufferSize;
    this.bufferPointer = 0;
    this.data = new Uint8Array(bufferSize);
    this.eventHandler = this.eventHandler.bind(this);
  }

  start(eventTarget = window.document) {
    if (this.eventTarget) {
      this.stop();
    }

    this.lastTime = +new Date();
    this.lastScreenX = Math.random() * window.innerHeight;
    this.lastScreenY = Math.random() * window.innerWidth;
    this.eventTarget = eventTarget;

    const eventHandler = this.eventHandler;
    eventTarget.addEventListener('keydown', eventHandler);
    eventTarget.addEventListener('keyup', eventHandler);
    eventTarget.addEventListener('mousedown', eventHandler);
    eventTarget.addEventListener('mouseup', eventHandler);
    eventTarget.addEventListener('touchstart', eventHandler);
    eventTarget.addEventListener('touchend', eventHandler);
  }

  stop() {
    const eventTarget = this.eventTarget;
    if (eventTarget) {
      return;
    }
    this.eventTarget = undefined;

    const eventHandler = this.eventHandler;
    eventTarget.addEventListener('keydown', eventHandler);
    eventTarget.addEventListener('keyup', eventHandler);
    eventTarget.addEventListener('mousedown', eventHandler);
    eventTarget.addEventListener('mouseup', eventHandler);
    eventTarget.addEventListener('touchstart', eventHandler);
    eventTarget.addEventListener('touchend', eventHandler);
  }

  eventHandler(event) {
    const time = event.timeStamp || +(new Date());
    const timeDiff = time - this.lastTime;
    if (timeDiff === 0) return;

    this.lastTime = time;
    this.registerEntropy(timeDiff);

    if (event.screenX && event.screenY) {
      this.positionEventHandler(event);
    }

    if (event.touches && event.touches.length > 0) {
      for (let i = 0, l = event.touches.length; i < l; i += 1) {
        this.positionEventHandler(event.touches[i]);
      }
    }
    // console.log(this.export());
  }

  positionEventHandler(event) {
    this.registerEntropy(this.lastScreenX - event.screenX);
    this.registerEntropy(this.lastScreenY - event.screenY);
    this.lastScreenX = event.screenX;
    this.lastScreenY = event.screenY;
  }

  registerEntropy(value) {
    // eslint-disable-next-line no-bitwise
    this.data[this.bufferPointer] ^= value;
    this.bufferPointer = (this.bufferPointer + 1) % this.bufferSize;
  }

  ping() {
    this.eventHandler({ timestamp: +new Date() });
  }

  export() {
    return btoa(Array.from(this.data).map((b) => String.fromCharCode(b)).join(''));
  }
}

let singleton;
export function getDefaultInstance() {
  if (!singleton) {
    singleton = new Entropy();
  }
  return singleton;
}

export default getDefaultInstance;
