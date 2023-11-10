library("tidyverse")

data <- read_csv("data.csv") %>%
  mutate(
    `TX Power [dBm]` = factor(`TX Power [dBm]`, levels = c(-12, -8, -4, 0, 4, 8)),
    `Sample Rate [Hz]` = factor(`Sample Rate [Hz]`, levels = c(2500, 5000))
  )

png(width = 8, height = 5, units = "in", res = 300)

ggplot(data) +
  aes(`TX Power [dBm]`, `Current [mA]`, colour = `Sample Rate [Hz]`) +
  geom_boxplot() +
  labs(
    title = "Average System Current"
  )

ggplot(data) +
  aes(`TX Power [dBm]`, `Peak Current [mA]`, colour = `Sample Rate [Hz]`) +
  geom_boxplot() +
  labs(
    title = "Peak System Current"
  )

ggplot(data) +
  aes(`TX Power [dBm]`, `Base Current [mA]`, colour = `Sample Rate [Hz]`) +
  geom_boxplot() +
  labs(
    title = "Baseline System Current"
  )
