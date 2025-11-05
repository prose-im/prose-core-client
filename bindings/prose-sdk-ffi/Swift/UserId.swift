import Foundation

public struct UserId:
  Codable,
  Hashable,
  Sendable,
  RawRepresentable,
  CustomStringConvertible,
  CustomDebugStringConvertible
{
  public let rawValue: String

  public init?(rawValue: String) {
    guard isValidUserId(userId: rawValue) else {
      return nil
    }
    self.rawValue = rawValue
  }

  public init?(_ value: String) {
    self.init(rawValue: value)
  }

  internal init(unsafe value: String) {
    self.rawValue = value
  }

  public var debugDescription: String {
    self.rawValue
  }

  public var description: String {
    self.rawValue
  }
}
