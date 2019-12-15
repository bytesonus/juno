using System.Collections.Generic;
using Gotham.Utils;
using Newtonsoft.Json;

namespace Gotham.Models
{
	public abstract class BasePacket
	{
		[JsonProperty(PropertyName = "requestId")]
		public string RequestID { get; internal set; }
		[JsonProperty(PropertyName = "type")]
		public string PacketType { get; internal set; }

		protected BasePacket(string requestId, string PacketType)
		{
			this.PacketType = PacketType;
			this.RequestID = requestId;
		}
	}

	public class ModuleRegistrationPacket : BasePacket
	{
		[JsonProperty(PropertyName = "moduleId")]
		public string ModuleID { get; internal set; } = "undefined";
		[JsonProperty(PropertyName = "version")]
		public string Version { get; internal set; } = "0.0.0";
		[JsonProperty(PropertyName = "dependencies")]
		public Dictionary<string, string> Dependencies { get; internal set; } = new Dictionary<string, string>();

		public ModuleRegistrationPacket(string requestId) : base(requestId, Constants.ModuleRegistered)
		{

		}
	}
}