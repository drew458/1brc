package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"sync"
)

const BUF_SIZE = 100000

type MeasurementAggregate struct {
	min   float32
	sum   float32
	count float32
	max   float32
}

func main() {
	procNum := runtime.NumCPU()
	linesChan := make(chan []string)

	go func(c chan<- []string) {
		f, err := os.Open("measurements.txt")
		if err != nil {
			log.Fatalf("Failed to open measurements file: %s", err)
		}
		defer f.Close()

		s := bufio.NewScanner(f)
		buf := make([]string, 0, BUF_SIZE)

		for s.Scan() {
			line := s.Text()

			if len(buf) < 100000 {
				buf = append(buf, line)
			} else {
				c <- buf
				buf = make([]string, 0, BUF_SIZE)
			}
		}

		c <- buf

		close(c)
	}(linesChan)

	var wg sync.WaitGroup
	mapsChan := make(chan map[string]MeasurementAggregate, procNum)

	for range procNum {
		wg.Add(1)
		go func(linesChan <-chan []string, mapsChan chan<- map[string]MeasurementAggregate) {
			defer wg.Done()

			m := make(map[string]MeasurementAggregate)

			for lineSlice := range linesChan {
				for _, line := range lineSlice {
					// Parse the text line to obtain city and temperature
					split := strings.Split(line, ";")
					city := split[0]
					val, err := strconv.ParseFloat(split[1], 32)
					if err != nil {
						log.Fatalf("Failed to read measurement from file: %s", err)
					}
					temperatureMeasurement := float32(val)

					// Update the min, mean, max
					var min float32
					var sum float32
					var count float32
					var max float32
					stats, found := m[city]
					if found {
						min = stats.min

						if temperatureMeasurement < min {
							min = temperatureMeasurement
						}

						count = stats.count + 1
						sum = sum + temperatureMeasurement

						max = stats.max
						if temperatureMeasurement > max {
							max = temperatureMeasurement
						}
					} else {
						min = temperatureMeasurement
						sum = temperatureMeasurement
						count = 1.0
						max = temperatureMeasurement
					}

					m[city] = MeasurementAggregate{min, sum, count, max}
				}
			}

			mapsChan <- m
		}(linesChan, mapsChan)
	}

	go func() {
		wg.Wait()
		close(mapsChan)
	}()

	m := make(map[string]MeasurementAggregate)
	// Takes the intermediate maps from mapsChan and assemble it into m, the global one
	for intermediateMap := range mapsChan {

		for city, temperatureMeasurement := range intermediateMap {
			// Update the global min, mean, max
			var globalMin float32
			var globalSum float32
			var globalCount float32
			var globalMax float32
			globalStats, found := m[city]
			if found {
				globalMin = globalStats.min
				if temperatureMeasurement.min < globalMin {
					globalMin = temperatureMeasurement.min
				}

				globalSum = globalStats.sum + temperatureMeasurement.sum
				globalCount = globalStats.count + 1

				globalMax = globalStats.max
				if temperatureMeasurement.max > globalMax {
					globalMax = temperatureMeasurement.max
				}
			} else {
				globalMin = temperatureMeasurement.min
				globalSum = temperatureMeasurement.sum
				globalCount = 1.0
				globalMax = temperatureMeasurement.max
			}

			m[city] = MeasurementAggregate{globalMin, globalSum, globalCount, globalMax}
		}
	}

	cities := make([]string, 0, len(m))
	finalMap := make(map[string]string)

	for city, measurements := range m {
		min := measurements.min
		avg := measurements.sum / measurements.count
		max := measurements.max

		finalMap[city] = fmt.Sprintf("%.1f/%.1f/%.1f", min, avg, max)
		cities = append(cities, city)
	}

	sort.Strings(cities)

	var sb strings.Builder
	sb.WriteString("{")
	for idx, city := range cities {
		sb.WriteString(fmt.Sprintf("%s=%s", city, finalMap[city]))

		if idx < len(cities)-1 {
			sb.WriteString(", ")
		}
	}
	sb.WriteString("}\n")

	fmt.Print(sb.String())
}
