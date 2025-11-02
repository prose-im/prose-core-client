import UIKit

extension UIColor {
    convenience init ? (prose_hex rgbString: String) {
        guard let rgb = rgbString.prose_RGBValue else {
            return nil
        }
        self.init(red: CGFloat(rgb.r), green: CGFloat(rgb.g), blue: CGFloat(rgb.b), alpha: 1)
    }
}

private extension String {
    /// Returns an RGB tuple by trying to parse the sender as a hex color (e.g. #ff00ff).
    /// The values in the returned tuple are in the range between 0 and 1.
    var prose_RGBValue: (r: Float, g: Float, b: Float)? {
        let str = self.hasPrefix("#") ? self.dropFirst(): self[...]

        guard let value = Int(str, radix: 16) else {
            return nil
        }

        return value.prose_RGBValue
    }
}

private extension Int {
    /// Interprets the receiver as a hexadecimal color value and returns its RGB values in a tuple.
    /// The values in the returned tuple are in the range between 0 and 1.
    var prose_RGBValue: (r: Float, g: Float, b: Float) {
        (r: Float((self >> 16) % 256) / 255,
        g: Float((self >> 8) % 256) / 255,
        b: Float((self >> 0) % 256) / 255)
    }
}
