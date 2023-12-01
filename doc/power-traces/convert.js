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
  return {
    samples: result,
    rate: Number(data.readBigUInt64LE(24))
  }
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

let csv = 'PHY,Channel Count,Sample Rate [Hz],TX Power [dBm],Current [mA],Base Current [mA],Peak Current [mA]\n'
phys.forEach(phy => {
  channelCounts.forEach(channelCount => {
    sampleRates.forEach(sampleRate => {
      let tsv = 'tx_power\taverage_current\n'
      txPowers.forEach(txPower => {
        const name = [
          sampleRate + 'sps',
          channelCount + 'ch',
          plusminus(txPower) + 'dbm',
          phy.toLowerCase(),
        ].join('-') + '.bin.gz'
        try {
          const {samples, rate} = loadFile(name)
          let windowSize = samples.length //rate / 2
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
            tsv += [
              txPower,
              (sum / windowSize / 68 * 1000).toFixed(2),
            ].join('\t') + '\n'
          }
          console.log('Processed ' + name)
        } catch (e) {
          console.error('Could not process ' + name)
          console.error(e)
        }
      })
      fs.writeFileSync([
        sampleRate + 'sps',
        channelCount + 'ch',
        phy.toLowerCase(),
      ].join('-') + '.dat', tsv)
    })
  })
})
fs.writeFileSync('data.csv', csv)
