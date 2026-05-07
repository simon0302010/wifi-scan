/* Taken from simon0302010/lswifi-freebsd */

#include <errno.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <net/if.h>
#include <sys/types.h>
#include <net/route.h>
#include <ifaddrs.h>
#include <net80211/ieee80211.h>
#include <net80211/ieee80211_netbsd.h>
#include <net80211/ieee80211_ioctl.h>
#include <sys/ioctl.h>
#include <sys/socket.h>

#define MAXWIFI 1536
#define MAXINTERFACES 256

static uint8_t wifi_interfaces = 0;

typedef struct {
    int io_s;
    const char *ifname;
} if_ctx;

typedef struct {
	char *interface;
	char *bssid;
	char *ssid;
	int rssi;
	uint16_t freq;
} lswifi_result;

static int scan_and_wait(if_ctx *ctx) {
    struct ieee80211req ireq;
    int sroute;

    sroute = socket(PF_ROUTE, SOCK_RAW, 0);
    if (sroute < 0) {
        perror("socket(PF_ROUTE,SOCK_RAW)");
        return -1;
    }
    memset(&ireq, 0, sizeof(ireq));
    strlcpy(ireq.i_name, ctx->ifname, sizeof(ireq.i_name));
    ireq.i_type = IEEE80211_IOC_SCAN_REQ;
    ireq.i_val = 0;

    if (ioctl(ctx->io_s, SIOCS80211, &ireq) == 0 || errno == EINPROGRESS) {
        wifi_interfaces++;

        char buf[2048];
        struct if_announcemsghdr *ifan;
        struct rt_msghdr *rtm;

        do {
            if (read(sroute, buf, sizeof(buf)) < 0) {
                perror("read(PF_ROUTE)");
                break;
            }
            rtm = (struct rt_msghdr *)(void *)buf;
            if (rtm->rtm_version != RTM_VERSION)
                break;
            ifan = (struct if_announcemsghdr *)rtm;
        } while (rtm->rtm_type != RTM_IEEE80211 ||
            ifan->ifan_what != RTM_IEEE80211_SCAN);
    } else if (errno == EINVAL || errno == ENOTTY) {
		close(sroute);
        return -1; // interface is not wifi
    } else {
        perror("ioctl");
		close(sroute);
        return -1;
    }
    close(sroute);
    // printf("scan completed\n");
    return 0;
}

static void mac_to_string(char buf[], const uint8_t mac[6]) {
    sprintf(buf, "%02x:%02x:%02x:%02x:%02x:%02x",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
}

static int get_scan_results(if_ctx *ctx, lswifi_result **networks, int *networks_idx) {
    uint8_t buf[24*1024];
    const uint8_t *cp;

    struct ieee80211req lenreq;
    memset(&lenreq, 0, sizeof(lenreq));
    strlcpy(lenreq.i_name, ctx->ifname, sizeof(lenreq.i_name));
    lenreq.i_type = IEEE80211_IOC_SCAN_RESULTS;
    lenreq.i_data = buf;
    lenreq.i_len = sizeof(buf);

    if (ioctl(ctx->io_s, SIOCG80211, &lenreq) < 0) {
        perror("ioctl");
        return -1;
    }
    if (lenreq.i_len < (int)sizeof(struct ieee80211req_scan_result)) {
        errno = EIO;
        return -1;
    }

    cp = buf;
    do {
        const struct ieee80211req_scan_result *sr = (const struct ieee80211req_scan_result *)(const void *)cp;
        const uint8_t *idp = (const uint8_t *)(sr + 1);

        char *bssid = malloc(24 * sizeof(char));
		if (bssid == NULL) {
			perror("malloc");
            return -1;
		}
        mac_to_string(bssid, sr->isr_bssid);

        char *ssid = malloc((IEEE80211_NWID_LEN + 1) * sizeof(char));
		if (ssid == NULL) {
            perror("malloc");
            free(bssid);
			return -1;
		}
        snprintf(ssid, IEEE80211_NWID_LEN + 1, "%.*s", sr->isr_ssid_len, idp);

        int rssi = sr->isr_rssi + sr->isr_noise;

        // printf("BSSID: %s, SSID: %s, FREQ: %u, RSSI: %i, CAPINFO: %u\n", bssid, ssid, sr->isr_freq, rssi, sr->isr_capinfo);

        lswifi_result *result = malloc(sizeof(lswifi_result));
        if (result == NULL) {
            free(ssid);
            free(bssid);
            return -1;
        } else {
			char *ifname = strdup(ctx->ifname);
            if (ifname == NULL) {
                perror("strdup");
                free(ssid);
                free(bssid);
                free(result);
                return -1;
            }
            *result = (lswifi_result){
                .interface = ifname,
                .ssid = ssid,
                .bssid = bssid,
                .rssi = rssi,
                .freq = sr->isr_freq,
            };

			if (*networks_idx < MAXWIFI) {
				networks[*networks_idx] = result;
            	(*networks_idx)++;
			} else {
				fprintf(stderr, "warning: more than %i networks have been detected\n", MAXWIFI);
				free(ssid);
				free(bssid);
				free(result);
				free(ifname);
				return 0;
			}
        }

        cp += sr->isr_len, lenreq.i_len -= sr->isr_len;
    } while (lenreq.i_len >= (int)sizeof(struct ieee80211req_scan_result));

    return 0;
}

void free_networks(lswifi_result **networks) {
    for (int i = 0; networks[i] != NULL; i++) {
        free(networks[i]->bssid);
        free(networks[i]->ssid);
        free(networks[i]->interface);
        free(networks[i]);
    }
    free(networks);
}

lswifi_result **get_networks() {
    struct ifaddrs *ifap;
    struct ifaddrs *ifa;

    if (getifaddrs(&ifap) != 0) {
        perror("getifaddrs");
        return NULL;
    }
 
    int io_s = socket(AF_INET, SOCK_DGRAM, 0);
	if (io_s == -1) {
		fprintf(stderr, "failed to open socket\n");
		exit(1);
	}
    lswifi_result **networks = malloc((MAXWIFI + 1) * sizeof(lswifi_result *));
	if (networks == NULL) {
		fprintf(stderr, "failed to allocate networks\n");
		exit(1);
	}
    int networks_idx = 0;

    wifi_interfaces = 0;
    const char *seen_interfaces[MAXINTERFACES] = {NULL};
    int seen_interfaces_idx = 0;

    for (ifa = ifap; ifa; ifa = ifa->ifa_next) {
        int found_iface = 0;
        for (int i = 0; i < seen_interfaces_idx; i++) {
            if (!strcmp(seen_interfaces[i], ifa->ifa_name)) {
                found_iface = 1;
                break;
            }
        }
        if (found_iface)
            continue;;

        if (seen_interfaces_idx < MAXINTERFACES) {
            seen_interfaces[seen_interfaces_idx] = ifa->ifa_name;
            seen_interfaces_idx++;
        }

        if_ctx ctx = {
            .ifname = ifa->ifa_name,
            .io_s = io_s
        };

        // printf("trying interface %s\n", ifa->ifa_name);

        if (scan_and_wait(&ctx) == 0) {
            if (get_scan_results(&ctx, networks, &networks_idx) != 0) {
                goto on_fail;
            }
        } else if (errno != EINVAL && errno != ENOTTY) {
            goto on_fail;
        }
    }

    if (wifi_interfaces == 0) {
        errno = ENXIO;
        goto on_fail;
    }

    freeifaddrs(ifap);
    networks[networks_idx] = NULL;
    close(io_s);
    return networks;

on_fail:
    freeifaddrs(ifap);
    networks[networks_idx] = NULL;
    close(io_s);
    free_networks(networks);
    return NULL;
}
