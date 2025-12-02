# linky-binding-rs

This is Linky Rust afb binding

## Source configuration

Linky binding support with direct serial as /dev/tty and network client UDP server mode

### cycle

For sensor event push even when data does not change. Default is 0 and event are only push when a sensor value changes.

```bash
# force event push every 25s
cycle: 25
```

### Network

default bind is 0.0.0.0 (all host interfaces). Port as no default value and should be define

```bash
network:
    - bind: 192.168.1.61
    port: 2000
```

### Serial

```bash
serial:
    - device: /dev/ttyUSB0
    speed: 9600
    parity: even
```

## Sensors config

Selected sensors create binding API verbs. Some sensor are read only when other are subscribable and send automatically event each time internal value change.

```yaml
sensors:
        # IINST:   # 60-90A meeting only
        # ADSC:    # meeting code addr
        MSG:       # short message
        NTARF:     # current tariff index
        ADPS:      # Avertissent depassement puissance
        TARIFF:    # Current Tariff name and label

        ENERGY:    # Total energy meeter

        SINSTS:    # Current used power
        SINSTI:    # Current injected power

        POWER-IN:  # Today time & max power
        POWER-OUT: # Yesterday time & max power

        STGE:      # Linky status register
        #DPM1:      # Mobile Pic start
        #FPM1:      # Mobile Pic end
```

## debug

### Serial Linky connectivity check

Depending on your configuration your Linky meeter may talk either 1200 or 9600 baud. In both case it uses parity:odd/7bits mode. To change from 1200 to 9600 contact your energy provider that may change meeter mode remotely.

```bash
picocom -b 9600 -d 7 -p o /dev/ttyUSB_TIC
```

### Network Linky connectivity check

socat is a simple way to check that your host accepts incoming UDP packets. If your packets are not 'human' readable then your USR-TCP232 serial speed/parity is probably not correctly configured.

```bash
socat - UDP4-LISTEN:2000
```

### Serial/TTL_to_UDP config

When using a USR-TCP232 device you should use USR-TCP232-Config.js to configure your device to point to your host/port UDP port (Warning: do not forget to open port on your firewall)

```javascript
# update ttl232_to_udp/USR-TCP232-Config.js with
var DST_ADDR="192.168.1.98";  // binding target IP addr
var DST_PORT=2000;            // binding listening port
var USR_MODE=0;               // mode client UDP
var USR_BAUD=9600;            // speed 1200 for Linky legacy & 9600 for new mode
var USR_STOP=2;               // mode 2=>"E,7,1" (parity:even, data:7bit, stopbit:1)
var USR_ADDR="192.168.1.32";  // unused nevertheless mandatory
var USR_GATW="192.168.1.1";   // ""
var USR_SNET="255.255.255.0"; // ""
```
