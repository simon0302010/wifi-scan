/* Taken from simon0302010/lswifi-freebsd */

#include <asm-generic/errno-base.h>
#include <errno.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <net/if.h>
#include <net/route.h>
#include <ifaddrs.h>
#include <net80211/ieee80211.h>
#include <net80211/ieee80211_freebsd.h>
#include <net80211/ieee80211_ioctl.h>
#include <lib80211/lib80211_ioctl.h>
#include <sys/ioctl.h>
#include <sys/socket.h>

#define MAXCHAN 1536
#define MAXWIFI 1536

static struct ieee80211req_chaninfo *chaninfo;

static uint8_t wifi_interfaces = 0;

typedef struct {
    int io_s;
    const char *ifname;
} if_ctx;

typedef struct {
	char *interface;
//	int connected;
	char *bssid;
	char *ssid;
	int rssi;
	int channel;
} lswifi_result;

static int scan_and_wait(if_ctx *ctx) {
    struct ieee80211_scan_req sr;
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

    memset(&sr, 0, sizeof(sr));
	sr.sr_flags = IEEE80211_IOC_SCAN_ACTIVE
		    | IEEE80211_IOC_SCAN_BGSCAN
		    | IEEE80211_IOC_SCAN_NOPICK
		    | IEEE80211_IOC_SCAN_ONCE;
    sr.sr_duration = IEEE80211_IOC_SCAN_FOREVER;
    sr.sr_nssid = 0;

    ireq.i_data = &sr;
    ireq.i_len = sizeof(sr);

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

static void getchaninfo(if_ctx *ctx) {
    if (chaninfo != NULL)
        return;
    chaninfo = malloc(IEEE80211_CHANINFO_SIZE(MAXCHAN));
    if (chaninfo == NULL) {
        fprintf(stderr, "no space for channel list\n");
        exit(1);
    }
    if (lib80211_get80211(ctx->io_s, ctx->ifname, IEEE80211_IOC_CHANINFO, chaninfo, IEEE80211_CHANINFO_SIZE(MAXCHAN)) < 0) {
        fprintf(stderr, "unable to get channel information\n");
        exit(1);
    }
}

static int freq_to_channel(struct ieee80211req_chaninfo *chaninfo, uint16_t freq) {
    for (int i = 0; i < chaninfo->ic_nchans; i++)
        if (chaninfo->ic_chans[i].ic_freq == freq)
            return chaninfo->ic_chans[i].ic_ieee;
    return 0;
}

static int get_scan_results(if_ctx *ctx, lswifi_result **networks, int *networks_idx) {
    uint8_t buf[24*1024];
    const uint8_t *cp;
    int len, idlen;

    if (lib80211_get80211len(ctx->io_s, ctx->ifname, IEEE80211_IOC_SCAN_RESULTS, buf, sizeof(buf), &len) < 0) {
        perror("lib80211_get80211len");
        return -1;
    }
    if (len < (int)sizeof(struct ieee80211req_scan_result)) {
        errno = EIO;
        return -1;
    }

    getchaninfo(ctx);

    cp = buf;
    do {
        const struct ieee80211req_scan_result *sr;
        const uint8_t *vp, *idp;

        sr = (const struct ieee80211req_scan_result *)(const void *)cp;
        vp = cp + sr->isr_ie_off;
        if (sr->isr_meshid_len) {
            idp = vp + sr->isr_ssid_len;
            idlen = sr->isr_meshid_len;
        } else {
            idp = vp;
            idlen = sr->isr_ssid_len;
        }

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
        snprintf(ssid, IEEE80211_NWID_LEN + 1, "%.*s", idlen, idp);

        int rssi = sr->isr_rssi + sr->isr_noise;

        int channel = freq_to_channel(chaninfo, sr->isr_freq);
        if (channel == 0)
            fprintf(stderr, "warning: could not find channel\n");

        // printf("BSSID: %s, SSID: %s, CHANNEL: %u, RSSI: %i, CAPINFO: %u\n", bssid, ssid, channel, rssi, sr->isr_capinfo);

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
                .channel = channel,
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

        cp += sr->isr_len, len -= sr->isr_len;
    } while (len >= (int)sizeof(struct ieee80211req_scan_result));

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

    for (ifa = ifap; ifa; ifa = ifa->ifa_next) {
        if_ctx ctx = {
            .ifname = ifa->ifa_name,
            .io_s = io_s
        };

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
