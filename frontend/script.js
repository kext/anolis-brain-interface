const ms = n => new Promise(resolve => setTimeout(resolve, n))
const pad = n => (n < 10 ? '0' : '') + n
const now = () => {
  let t = new Date()
  return t.getFullYear() + pad(t.getMonth() + 1) + pad(t.getDate()) + '-' + pad(t.getHours()) + pad(t.getMinutes())
}
const app = Vue.createApp({
  data() {
    return {
      device: null,
      start: 0,
      transferred: 0,
      plots: [],
      running: false,
      recording: [],
      recordingSize: 0
    }
  },
  computed: {
    speed() {
      if (this.transferred > 0) {
        return (this.transferred / (Date.now() - this.start)).toFixed(1) + 'kB/s'
      } else {
        return '-'
      }
    }
  },
  methods: {
    liveViewFrame(frame) {
      const limit = 500
      if (frame.length !== this.plots.length) {
        this.clearPlots(frame.length)
      }
      for (let i = 0; i < this.plots.length; ++i) {
        const d = this.plots[i].value.data
        d.push(frame[i])
        while (d.length > limit) {
          d.shift()
        }
      }
    },
    clearPlots(count) {
      let plots = []
      for (let i = 0; i < count; ++i) {
        plots.push(Vue.shallowRef({
          name: 'Channel ' + (i + 1),
          color: `oklch(69% 0.15 ${Math.round(360 * i / count)})`,
          data: []
        }))
      }
      this.plots = plots
    },
    async deviceLoop(device) {
      try {
        await device.open()
        await device.selectConfiguration(1)
        await device.claimInterface(0)
        this.device = device
        this.send()
        while (true) {
          let d = await device.transferIn(1, 2048)
          if (d.status === 'ok') {
            this.transferred += d.data.byteLength
            this.recordPacket(d.data)
            if (d.data.byteLength > 2) {
              let channels = d.data.getUint8(1)
              let frame = []
              for (let i = 0; i < Math.floor((d.data.byteLength - 2) / 2); ++i) {
                let v = (d.data.getUint16(2 * i + 2, true) - 32768) / 32768
                if (frame.length < channels) {
                  frame.push([v, v])
                } else {
                  let m = frame[i % channels]
                  if (v < m[0]) m[0] = v
                  if (v > m[1]) m[1] = v
                }
              }
              this.liveViewFrame(frame)
            }
          }
        }
      } catch (e) {
        console.error(e)
        this.device = null
      }
    },
    connect() {
      navigator.usb.requestDevice({
        filters: [
          {
            vendorId: 0xbf50,
            productId: 0x0b70
          }
        ]
      })
      .then(async device => {
        this.deviceLoop(device)
      })
    },
    async send() {
      this.start = Date.now()
      this.transferred = 0
      while (true) {
        if (this.running) {
          await this.device.transferOut(1, new Uint8Array([1,2,3,4]))
        }
        await ms(100)
      }
    },
    recordPacket(packet) {
      const header = new DataView(new ArrayBuffer(16))
      header.setUint32(0, 0x55daba, true)
      header.setUint32(4, header.byteLength + packet.byteLength, true)
      header.setBigUint64(8, BigInt(Date.now()), true)
      this.recording.push(header, packet)
      this.recordingSize += header.byteLength + packet.byteLength
    },
    save() {
      const blob = new Blob(this.recording, { type: 'application/octet-stream' })
      const a = document.createElement('a')
      a.href = URL.createObjectURL(blob)
      a.download = `brain-recording-${now()}.bin`
      setTimeout(() => {
        URL.revokeObjectURL(blob)
      }, 1000)
      a.click()
    },
    formatSize(bytes) {
      if (bytes < 1000) {
        return bytes + 'B'
      } else if (bytes < 1e6) {
        return (bytes / 1000).toFixed(5 - Math.floor(Math.log10(bytes))) + 'kB'
      } else if (bytes < 1e9) {
        return (bytes / 1e6).toFixed(8 - Math.floor(Math.log10(bytes))) + 'MB'
      } else {
        return (bytes / 1e9).toFixed(11 - Math.floor(Math.log10(bytes))) + 'GB'
      }
    }
  },
  mounted() {
    navigator.usb.getDevices().then(devices => {
      if (devices.length > 0) {
        this.deviceLoop(devices[0])
      }
    })
  }
})
.use(plot)
.use(icons)
.mount('body')
window.addEventListener('keydown', e => {
  if (e.key === 'f') {
    if (document.fullscreenElement) {
      document.exitFullscreen()
    } else {
      document.querySelector('html').requestFullscreen()
    }
  }
})
