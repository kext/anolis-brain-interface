const fs = require('fs')
const zlib = require('zlib')

const loadFile = name => {
  const data = zlib.gunzipSync(fs.readFileSync(name))
  if (new TextDecoder().decode(data.subarray(0, 8)) !== '<SALEAE>') throw new Error('Invalid signature')
  if (data.readInt32LE(8) != 0) throw new Error('Wrong version')
  if (data.readInt32LE(12) != 1) throw new Error('Expected analogue trace')
  const samples = Number(data.readBigUInt64LE(40))
  const result = new Float32Array(samples)
  for (let i = 0; i < samples; ++i) {
    result[i] = data.readFloatLE(i * 4 + 48)
  }
  return result
}

const plusminus = n => {
  if (n < 0) return 'minus' + (-n)
  if (n > 0) return 'plus' + n
  return 'zero'
}

const phys = ['2M']
const channelCounts = [8]
const sampleRates = [2500, 5000]
const txPowers = [-12, -8, -4, 0, 4, 8]
const windowSize = 750e3

let csv = 'PHY,Channel Count,Sample Rate [Hz],TX Power [dBm],Current [mA],Base Current [mA],Peak Current [mA]\n'
phys.forEach(phy => {
  channelCounts.forEach(channelCount => {
    sampleRates.forEach(sampleRate => {
      txPowers.forEach(txPower => {
        const name = [
          sampleRate + 'sps',
          channelCount + 'ch',
          plusminus(txPower) + 'dbm',
          phy.toLowerCase(),
        ].join('-') + '.bin.gz'
        try {
          const samples = loadFile(name)
          let n = Math.floor(samples.length / windowSize)
          for (let i = 0; i < n; ++i) {
            let sum = 0
            let min = samples[0]
            let max = samples[0]
            for (let j = 0; j < windowSize; ++j) {
              let s = samples[i * windowSize + j]
              sum += s
              if (s < min) min = s
              if (s > max) max = s
            }
            csv += [
              phy, channelCount, sampleRate, txPower,
              (sum / windowSize / 68 * 1000).toFixed(2),
              (min / 68 * 1000).toFixed(2),
              (max / 68 * 1000).toFixed(2),
            ].join(',') + '\n'
          }
          console.log('Processed ' + name)
        } catch (e) {
          console.error('Could not process ' + name)
          console.error(e)
        }
      })
    })
  })
})
fs.writeFileSync('data.csv', csv)
