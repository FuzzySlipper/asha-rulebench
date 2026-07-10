const target = process.env.RULEBENCH_HOST_URL;

if (typeof target !== 'string' || target.length === 0) {
  throw new Error('RULEBENCH_HOST_URL is required by the Rulebench proxy.');
}

module.exports = {
  '/api/rulebench': {
    target,
    secure: false,
    changeOrigin: false,
  },
};
