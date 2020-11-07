/**
 *  Fork of https://github.com/catdad/canvas-confetti
 * ISC license.
 * See https://github.com/catdad/canvas-confetti/blob/master/LICENSE
 * This is a library not using the general Norgance license.
 *
 * Without workers and stripped down.
 * */
/* eslint-enable no-param-reassign */
const defaults = {
  particleCount: 50,
  angle: 90,
  spread: 45,
  startVelocity: 45,
  decay: 0.9,
  gravity: 1,
  ticks: 200,
  x: 0.5,
  y: 0.5,
  shapes: ['square', 'circle'],
  zIndex: 100,
  colors: [
    '#26ccff',
    '#a25afd',
    '#ff5e7e',
    '#88ff5a',
    '#fcff42',
    '#ffa62d',
    '#ff36ff',
  ],
  // probably should be true, but back-compat
  disableForReducedMotion: false,
  scalar: 1,
};

const TWO_PI = Math.PI * 2;
const TENTH_PI = Math.PI / 10;
const SIXTY_HZ_IN_MS = 1000.0 / 60.0;

function convert(val, transform) {
  return transform ? transform(val) : val;
}

function isOk(val) {
  return !(val === null || val === undefined);
}

function prop(options, name, transform) {
  return convert(
    options && isOk(options[name]) ? options[name] : defaults[name],
    transform,
  );
}

function onlyPositiveInt(number) {
  return number < 0 ? 0 : Math.floor(number);
}

function randomInt(min, max) {
  // [min, max)
  return Math.floor(Math.random() * (max - min)) + min;
}

function toDecimal(str) {
  return parseInt(str, 16);
}

function hexToRgb(str) {
  let val = String(str).replace(/[^0-9a-f]/gi, '');

  if (val.length < 6) {
    val = val[0] + val[0] + val[1] + val[1] + val[2] + val[2];
  }

  return {
    r: toDecimal(val.substring(0, 2)),
    g: toDecimal(val.substring(2, 4)),
    b: toDecimal(val.substring(4, 6)),
  };
}

function getOrigin(options) {
  const origin = prop(options, 'origin', Object);
  origin.x = prop(origin, 'x', Number);
  origin.y = prop(origin, 'y', Number);

  return origin;
}

function setCanvasWindowSize(canvas) {
  // eslint-disable-next-line no-param-reassign
  canvas.width = document.documentElement.clientWidth;
  // eslint-disable-next-line no-param-reassign
  canvas.height = document.documentElement.clientHeight;
}

function setCanvasRectSize(canvas) {
  const rect = canvas.getBoundingClientRect();
  // eslint-disable-next-line no-param-reassign
  canvas.width = rect.width;
  // eslint-disable-next-line no-param-reassign
  canvas.height = rect.height;
}

function getCanvas(zIndex) {
  const canvas = document.createElement('canvas');

  canvas.style.position = 'fixed';
  canvas.style.top = '0px';
  canvas.style.left = '0px';
  canvas.style.pointerEvents = 'none';
  canvas.style.zIndex = zIndex;

  return canvas;
}

function randomPhysics(opts) {
  const radAngle = opts.angle * (Math.PI / 180);
  const radSpread = opts.spread * (Math.PI / 180);

  return {
    x: opts.x,
    y: opts.y,
    wobble: Math.random() * 10,
    velocity: (opts.startVelocity * 0.5) + (Math.random() * opts.startVelocity),
    angle2D: -radAngle + ((0.5 * radSpread) - (Math.random() * radSpread)),
    tiltAngle: Math.random() * Math.PI,
    color: hexToRgb(opts.color),
    shape: opts.shape,
    tick: 0,
    totalTicks: opts.ticks,
    decay: opts.decay,
    random: Math.random() + 5,
    tiltSin: 0,
    tiltCos: 0,
    wobbleX: 0,
    wobbleY: 0,
    gravity: opts.gravity * 3,
    ovalScalar: 0.6,
    scalar: opts.scalar,
  };
}

function updateFetti(context, fetti, timeRatio) {
  /* eslint-disable no-param-reassign */
  fetti.x += Math.cos(fetti.angle2D) * fetti.velocity * timeRatio;
  fetti.y += Math.sin(fetti.angle2D) * fetti.velocity * timeRatio + fetti.gravity * timeRatio;
  fetti.wobble += 0.1 * timeRatio;
  fetti.velocity *= 1 - ((1 - fetti.decay) * timeRatio);
  fetti.tiltAngle += 0.1 * timeRatio;
  fetti.tiltSin = Math.sin(fetti.tiltAngle);
  fetti.tiltCos = Math.cos(fetti.tiltAngle);
  fetti.random = Math.random() + 5;
  fetti.wobbleX = fetti.x + ((10 * fetti.scalar) * Math.cos(fetti.wobble));
  fetti.wobbleY = fetti.y + ((10 * fetti.scalar) * Math.sin(fetti.wobble));

  const progress = fetti.tick / fetti.totalTicks;
  fetti.tick += timeRatio;

  const x1 = fetti.x + (fetti.random * fetti.tiltCos);
  const y1 = fetti.y + (fetti.random * fetti.tiltSin);
  const x2 = fetti.wobbleX + (fetti.random * fetti.tiltCos);
  const y2 = fetti.wobbleY + (fetti.random * fetti.tiltSin);

  context.fillStyle = `rgba(${fetti.color.r}, ${fetti.color.g}, ${fetti.color.b}, ${1 - progress})`;
  context.beginPath();

  if (fetti.shape === 'circle') {
    context.ellipse(
      fetti.x,
      fetti.y,
      Math.abs(x2 - x1) * fetti.ovalScalar,
      Math.abs(y2 - y1) * fetti.ovalScalar,
      TENTH_PI * fetti.wobble,
      0,
      TWO_PI,
    );
  } else {
    context.moveTo(Math.floor(fetti.x), Math.floor(fetti.y));
    context.lineTo(Math.floor(fetti.wobbleX), Math.floor(y1));
    context.lineTo(Math.floor(x2), Math.floor(y2));
    context.lineTo(Math.floor(x1), Math.floor(fetti.wobbleY));
  }

  context.closePath();
  context.fill();

  return fetti.tick < fetti.totalTicks;
}

function animate(canvas, fettis, resizer, size, done) {
  let animatingFettis = fettis.slice();
  const context = canvas.getContext('2d');
  let animationFrame;
  let destroy;

  const prom = new Promise((resolve) => {
    function onDone() {
      animationFrame = null;
      destroy = null;

      context.clearRect(0, 0, size.width, size.height);

      done();
      resolve();
    }

    let lastUpdate;
    function update(timestamp) {
      if (lastUpdate === undefined) {
        lastUpdate = timestamp;
        animationFrame = requestAnimationFrame(update);
        return;
      }

      const diff = timestamp - lastUpdate;
      lastUpdate = timestamp;

      if (!size.width && !size.height) {
        resizer(canvas);
        size.width = canvas.width;
        size.height = canvas.height;
      }

      context.clearRect(0, 0, size.width, size.height);

      const timeRatio = diff / SIXTY_HZ_IN_MS;
      // console.log(timeRatio);
      let nbGoneFettis = 0;
      for (let i = 0, l = animatingFettis.length; i < l; i += 1) {
        const fetti = animatingFettis[i];
        if (fetti) {
          if (!updateFetti(context, fetti, timeRatio)) {
            nbGoneFettis += 1;
            animatingFettis[i] = undefined;
          }
        } else {
          nbGoneFettis += 1;
        }
      }
      if (nbGoneFettis > 64) {
        animatingFettis = animatingFettis.filter((fetti) => !!fetti);
      }

      if (animatingFettis.length) {
        animationFrame = requestAnimationFrame(update);
      } else {
        onDone();
      }
    }

    animationFrame = requestAnimationFrame(update);
    destroy = onDone;
  });

  return {
    addFettis(newFettis) {
      animatingFettis = animatingFettis.concat(newFettis);

      return prom;
    },
    canvas,
    promise: prom,
    reset() {
      if (animationFrame) {
        cancelAnimationFrame(animationFrame);
      }

      if (destroy) {
        destroy();
      }
    },
  };
}

export function confettiCannon(canvas, globalOpts) {
  const isLibCanvas = !canvas;
  const allowResize = !!prop(globalOpts || {}, 'resize');
  const globalDisableForReducedMotion = prop(globalOpts, 'disableForReducedMotion', Boolean);
  const resizer = isLibCanvas ? setCanvasWindowSize : setCanvasRectSize;
  let initialized = false;
  const preferLessMotion = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion)').matches;
  let animationObj;

  function fireLocal(options, size, done) {
    const particleCount = prop(options, 'particleCount', onlyPositiveInt);
    const angle = prop(options, 'angle', Number);
    const spread = prop(options, 'spread', Number);
    const startVelocity = prop(options, 'startVelocity', Number);
    const decay = prop(options, 'decay', Number);
    const gravity = prop(options, 'gravity', Number);
    const colors = prop(options, 'colors');
    const ticks = prop(options, 'ticks', Number);
    const shapes = prop(options, 'shapes');
    const scalar = prop(options, 'scalar');
    const origin = getOrigin(options);

    const fettis = [];

    const startX = canvas.width * origin.x;
    const startY = canvas.height * origin.y;

    for (let temp = particleCount; temp; temp -= 1) {
      fettis.push(
        randomPhysics({
          x: startX,
          y: startY,
          angle,
          spread,
          startVelocity,
          color: colors[temp % colors.length],
          shape: shapes[randomInt(0, shapes.length)],
          ticks,
          decay,
          gravity,
          scalar,
        }),
      );
    }

    // if we have a previous canvas already animating,
    // add to it
    if (animationObj) {
      return animationObj.addFettis(fettis);
    }

    animationObj = animate(canvas, fettis, resizer, size, done);

    return animationObj.promise;
  }

  function fire(options) {
    const disableForReducedMotion = globalDisableForReducedMotion || prop(options, 'disableForReducedMotion', Boolean);
    const zIndex = prop(options, 'zIndex', Number);

    if (disableForReducedMotion && preferLessMotion) {
      return new Promise((resolve) => {
        resolve();
      });
    }

    if (isLibCanvas && animationObj) {
      // use existing canvas from in-progress animation
      canvas = animationObj.canvas;
    } else if (isLibCanvas && !canvas) {
      // create and initialize a new canvas
      canvas = getCanvas(zIndex);
      document.body.appendChild(canvas);
    }

    if (allowResize && !initialized) {
      // initialize the size of a user-supplied canvas
      resizer(canvas);
    }

    const size = {
      width: canvas.width,
      height: canvas.height,
    };

    initialized = true;

    function onResize() {
      // don't actually query the size here, since this
      // can execute frequently and rapidly
      size.width = null;
      size.height = null;
    }

    function done() {
      animationObj = null;

      if (allowResize) {
        global.removeEventListener('resize', onResize);
      }

      if (isLibCanvas && canvas) {
        document.body.removeChild(canvas);
        canvas = null;
        initialized = false;
      }
    }

    if (allowResize) {
      global.addEventListener('resize', onResize, false);
    }

    return fireLocal(options, size, done);
  }

  fire.reset = () => {
    if (animationObj) {
      animationObj.reset();
    }
  };

  return fire;
}

const DEFAULT_CONFETTI = confettiCannon(null, { resize: true });
export default DEFAULT_CONFETTI;

export function fancyConfettiFromElement(element, opts) {
  const prout = confettiCannon(null, { resize: true });
  const {
    top, height, left, width,
  } = element.getBoundingClientRect();

  const y = (top + height / 2) / window.innerHeight;
  const x = (left + width / 2) / window.innerWidth;
  const origin = { x, y };

  const CONFETTI_COUNT = 200;

  [{
    particleRatio: 0.25,
    spread: 26,
    startVelocity: 55,
  }, {
    particleRatio: 0.2,
    spread: 60,
  }, {
    particleRatio: 0.35,
    spread: 100,
    decay: 0.91,
    scalar: 0.8,
  }, {
    particleRatio: 0.1,
    spread: 120,
    startVelocity: 25,
    decay: 0.92,
    scalar: 1.2,
  }, {
    particleRatio: 0.1,
    spread: 120,
    startVelocity: 45,
  },
  ].forEach((fire) => {
    // DEFAULT_CONFETTI({
    prout({
      origin,
      ...fire,
      ...opts,
      particleCount: Math.floor(CONFETTI_COUNT * fire.particleRatio),
    });
  });
}
