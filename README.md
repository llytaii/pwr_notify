### Installation
- configure args in pwr_notify.service as needed
- copy pwr_notify.service file in ~/.config/systemd/user/
- systemctl --user daemon-reload
- systemctl --user enable --now pwr_notify.service
