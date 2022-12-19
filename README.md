# userspace HID++

HID++ is a Logitech protocol. It is based on HID.

This library implements support for HID++ similar to the Linux kernel.
Support for HID++ 1.0 is currently **excluded**.

Implements the following HID++ 2.0+ features:

| ID     | Name | Spec | Supported version |
| ---    | ---  | ---  | ---               |
| `0x0000` | Root | [pdf](https://drive.google.com/file/d/1ULmw9uJL8b8iwwUo5xjSS9F5Zvno-86y/view?usp=share_link) | v2 |
| `0x0005` | DeviceTypeName | [pdf](https://drive.google.com/file/d/1V9UO0ToIIsMxhM36dEVfwkCFx4WH5qt2/view?usp=share_link) | v2 |
| `0x1990` | IlluminationLight | [pdf](https://drive.google.com/file/d/1SvD03KHG74C9TL2Dj3hwpBzVw-nSNA4l/view?usp=share_link) | v0 |

## Motivitation

In most Linux distributions older kernels are shipped.
To still be able to use these on older kernels, this library reimplements
the functionality for userspace.

## License

Since code is a rewrite of Linux kernel code, it only seems prudent to license
under same GPL2 license.
