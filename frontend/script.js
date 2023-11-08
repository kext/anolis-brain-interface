const ms = n => new Promise(resolve => setTimeout(resolve, n))
const app = Vue.createApp({
  data() {
    return {
      device: null,
      start: 0,
      transferred: 0,
      plots: []
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
      //const colors = ['#09d', '#c12', '#fa2', '#8a2']
      const colors = ['#09d']
      let plots = []
      for (let i = 0; i < count; ++i) {
        plots.push(Vue.shallowRef({
          name: 'Channel ' + (i + 1),
          color: colors[i % colors.length],
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
        await this.device.transferOut(1, new Uint8Array([1,2,3,4]))
        await ms(100)
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
