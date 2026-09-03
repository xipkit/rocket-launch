// Command ground-control talks to the pad controller and prints the state
// of every subsystem an operator cares about before a launch.
package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"net/http"
	"os"
	"sort"
	"sync"
	"text/tabwriter"
	"time"
)

// Subsystem is one row of the go/no-go poll.
type Subsystem struct {
	Name    string    `json:"name"`
	Owner   string    `json:"owner"`
	Go      bool      `json:"go"`
	Reason  string    `json:"reason,omitempty"`
	Updated time.Time `json:"updated"`
}

type client struct {
	base string
	http *http.Client
}

func (c *client) poll(ctx context.Context, name string) (Subsystem, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.base+"/subsystems/"+name, nil)
	if err != nil {
		return Subsystem{}, err
	}
	resp, err := c.http.Do(req)
	if err != nil {
		return Subsystem{}, fmt.Errorf("poll %s: %w", name, err)
	}
	defer resp.Body.Close()
	var s Subsystem
	if err := json.NewDecoder(resp.Body).Decode(&s); err != nil {
		return Subsystem{}, fmt.Errorf("decode %s: %w", name, err)
	}
	return s, nil
}

func main() {
	base := flag.String("pad", "http://pad.lc7.local:8080", "pad controller URL")
	timeout := flag.Duration("timeout", 3*time.Second, "per-subsystem poll timeout")
	flag.Parse()

	if flag.Arg(0) != "status" {
		fmt.Fprintln(os.Stderr, "usage: ground-control [flags] status")
		os.Exit(2)
	}

	names := []string{"sequencer", "telemetry", "trajectory", "range", "weather", "fts"}
	c := &client{base: *base, http: &http.Client{Timeout: *timeout}}
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	var (
		mu      sync.Mutex
		results []Subsystem
		wg      sync.WaitGroup
	)
	for _, name := range names {
		wg.Go(func() {
			s, err := c.poll(ctx, name)
			if err != nil {
				s = Subsystem{Name: name, Go: false, Reason: err.Error(), Updated: time.Now()}
			}
			mu.Lock()
			results = append(results, s)
			mu.Unlock()
		})
	}
	wg.Wait()

	sort.Slice(results, func(i, j int) bool { return results[i].Name < results[j].Name })
	w := tabwriter.NewWriter(os.Stdout, 0, 4, 2, ' ', 0)
	fmt.Fprintln(w, "SUBSYSTEM\tOWNER\tSTATE\tNOTE")
	allGo := true
	for _, s := range results {
		state := "GO"
		if !s.Go {
			state, allGo = "NO-GO", false
		}
		fmt.Fprintf(w, "%s\t%s\t%s\t%s\n", s.Name, s.Owner, state, s.Reason)
	}
	w.Flush()
	if !allGo {
		os.Exit(1)
	}
}
