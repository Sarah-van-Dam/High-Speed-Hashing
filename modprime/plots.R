library(tidyverse)
library(ggplot2)
library(readr)
library(stringr)

data <- read_csv("tmp.csv")

data.grouped <- data %>%
  group_by(scheme, bits, is128) %>%
  summarise(nspervalue = mean(nspervalue))

ggplot(data.grouped) +
  geom_point(aes(x = bits, y = nspervalue, color = scheme, shape = is128), size = 2) +
  scale_x_continuous(name = "bit-length of input", breaks = 32*(0:4), limits = c(0, 128)) +
  scale_y_continuous(name = "time per value [ns]", limits = c(0, 15))
