const { platform, arch } = process;

const platformArchMap = {
  darwin: {
    arm64: '@turndown-node/darwin-arm64',
  },
  linux: {
    arm64: '@turndown-node/linux-arm64-gnu',
    x64: '@turndown-node/linux-x64-gnu',
  },
  win32: {
    x64: '@turndown-node/win32-x64-msvc',
  },
};

function loadNativeBinding() {
  const packageName = platformArchMap[platform]?.[arch];

  if (!packageName) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
      `Supported: darwin-arm64, linux-x64, linux-arm64, win32-x64. ` +
      `Please open an issue at https://github.com/anthropics/turndown-node/issues`
    );
  }

  try {
    return require(packageName);
  } catch (e) {
    throw new Error(
      `Failed to load native binding for ${platform}-${arch}.\n` +
      `Package: ${packageName}\n` +
      `Error: ${e.message}\n\n` +
      `Try reinstalling with: npm install turndown-node`
    );
  }
}

const nativeBinding = loadNativeBinding();

module.exports = nativeBinding.TurndownService;
module.exports.TurndownService = nativeBinding.TurndownService;
module.exports.default = nativeBinding.TurndownService;
