library(tidyverse)
library(ggplot2)
library(readr)
library(stringr)

data1 <- read_csv("output/experiment1.csv")

data1.1 <- data1 %>%
  filter(bits <= 128)

ggplot(data1.1) +
  geom_point(size = 2, aes(x = bits, y = nanos, color = scheme, shape = is128)) +
  scale_x_continuous(name = "bit-length of input", breaks = 32*(0:4), limits = c(0, 128)) +
  scale_y_continuous(name = "time per value [ns]", limits = c(0, 80))

data1.2 <- data1 %>%
  filter(scheme %in% c("mmp", "mmp-triple", "shift", "shift-strong"))

ggplot(data1.2) +
  geom_point(size = 2, aes(x = bits, y = nanos, color = scheme, shape = is128)) +
  scale_x_continuous(name = "bit-length of input", breaks = 32*(0:4), limits = c(0, 128)) +
  scale_y_continuous(name = "time per value [ns]", limits = c(0, 15))

data1.3 <- data1 %>%
  filter(!(scheme %in% c("mmp", "mmp-triple", "shift", "shift-strong")))

ggplot(data1.3) +
  geom_point(size = 2, aes(x = bits, y = nanos, color = scheme, shape = is128)) +
  scale_x_continuous(name = "bit-length of input", breaks = 1024*(0:3), limits = c(0, 3072)) +
  scale_y_continuous(name = "time per value [ns]", limits = c(0, 800))


data2 <- read_csv("output/experiment2.csv")

ggplot(data2) +
  geom_point(size = 2, aes(x = family, y = nanos)) +
  scale_y_continuous(name = "time per byte [ns]", limits = c(0, 3))
