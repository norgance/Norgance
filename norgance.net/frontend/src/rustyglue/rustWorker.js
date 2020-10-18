/* eslint-disable no-restricted-globals */

let rust;
self.addEventListener('message', async (event) => {
  const {
    messageId,
    functionName,
    args,
    preload,
    className,
    freeResponseImmediately,
    returnClassName,
  } = event.data;

  if (typeof messageId === 'undefined') {
    return;
  }

  try {
    // Lazy loading
    if (!rust) {
      rust = await import('../../rust/pkg');
    }

    let rustFunction;

    if (className) {
      const ptr = event.data.ptr;
      if (!ptr) {
        throw new Error('Missing ptr');
      }
      const classConstructor = rust[className];
      if (!classConstructor || !classConstructor.constructor) {
        throw new TypeError(`Unknown class ${className}`);
      }

      // eslint-disable-next-line no-underscore-dangle
      const classInstance = classConstructor.__wrap(ptr);

      rustFunction = classInstance[functionName];
    } else {
      rustFunction = rust[functionName];
    }

    if (typeof rustFunction !== 'function') {
      throw new TypeError(`Unknown function ${functionName}`);
    }

    const convertedArgs = args.map((arg) => {
      if (arg && arg.className) {
        if (!arg.ptr) {
          throw new Error(`No valid ptr for ${arg.className}`);
        }
        const classConstructor = rust[arg.className];
        if (!classConstructor || !classConstructor.constructor) {
          throw new TypeError(`Unknown class ${arg.className}`);
        }
        // eslint-disable-next-line no-underscore-dangle
        return classConstructor.__wrap(arg.ptr);
      }
      return arg;
    });

    let returnObject = rustFunction(...convertedArgs);
    let transferables;

    if (returnObject.ptr && returnObject.constructor) {
      const finalObject = {
        ptr: returnObject.ptr,
        className: returnClassName,
      };

      if (preload) {
        Object.entries(preload).forEach(([key, {
          functionName: classFunctionName,
          args: classArgs = [],
        }]) => {
          const classRustFunction = returnObject[classFunctionName];
          if (typeof classRustFunction !== 'function') {
            throw new TypeError(`Unknown class function ${classFunctionName}`);
          }
          const value = classRustFunction.call(returnObject, classArgs);

          let buffer;
          if (value instanceof ArrayBuffer) {
            buffer = value;
          } else if (value && value.buffer instanceof ArrayBuffer) {
            buffer = value.buffer;
          }
          if (!transferables) {
            transferables = [buffer];
          } else {
            transferables.push(buffer);
          }
          finalObject[key] = value;
        });
      }

      if (freeResponseImmediately) {
        returnObject.free();
        finalObject.ptr = 0;
      }

      returnObject = finalObject;
    } else if (returnObject instanceof ArrayBuffer) {
      transferables = [returnObject];
    } else if (returnObject.buffer instanceof ArrayBuffer) {
      transferables = [returnObject.buffer];
    }

    self.postMessage({
      messageId,
      response: returnObject,
    }, transferables);
  } catch (error) {
    console.error(error);
    self.postMessage({
      messageId,
      error,
    });
  }
});
