# Security Policy

## Supported Versions

Currently supported versions with security updates:

| Version  | Supported          | Notes                                |
| -------- | ------------------ | ------------------------------------ |
| latest   | :white_check_mark: | Latest stable Docker image           |
| unstable | :warning:          | Development builds - use at own risk |
| v1.x.x   | :white_check_mark: | Tagged releases (when available)     |
| < v1.0   | :x:                | Pre-release versions not supported   |

## Security Considerations

### AI-Generated Code Notice

⚠️ **Important**: This project contains code generated with AI assistance (GitHub Copilot).
While thoroughly reviewed and tested, users should:

- Conduct additional security reviews before production deployment
- Validate all authentication and network security implementations
- Test thoroughly in isolated environments first
- Monitor for unusual behavior in production environments

### Network Security

This application exposes several network services:

- **ONVIF Service** (Port 8080 By Default) - HTTP/SOAP endpoints with authentication
- **RTSP Server** (Port 8554 By Default) - Media streaming service
- **WS-Discovery** (UDP 3702) - Multicast device discovery protocol (if enabled)

### Authentication Security

The project implements multiple authentication methods:

- **HTTP Basic Authentication** - Credentials sent base64 encoded
- **HTTP Digest Authentication** - Challenge-response mechanism
- **WS-Security** - SOAP security with PasswordDigest/PasswordText support

**Security Recommendations:**

- Change default credentials (`admin`/`onvif-rust`) in production
- Use strong passwords for ONVIF authentication
- Consider network-level security (VPNs, firewalls) for sensitive deployments

### Container Security

- Automatic vulnerability scanning with Trivy in CI/CD pipeline
- Regular dependency updates through automated workflows
- Non-root user execution where possible

## Reporting a Vulnerability

If you discover a security vulnerability within this project, please report it responsibly:

Use the [GitHub Security Advisory](https://github.com/W4ff1e/onvif-media-transcoder/security/advisories) feature:

### Preferred Method

1. Go to the Security tab in the GitHub repository
2. Click "Report a vulnerability"
3. Fill out the security advisory form
4. Provide detailed information about the vulnerability, including:
   - Steps to reproduce
   - Impact assessment
   - Suggested mitigations or fixes

### Response Timeline

- **Initial Response**: Within 48 hours of report
- **Assessment**: Within 1 week for severity evaluation
- **Fix Development**: Timeline depends on severity and complexity
- **Public Disclosure**: After fix is available and deployed

## Security Best Practices for Users

### Production Deployment

1. **Change Default Credentials**

   ```bash
   -e ONVIF_USERNAME="your-secure-username"
   -e ONVIF_PASSWORD="your-strong-password"
   ```

2. **Network Security**
   - Use firewalls to restrict access to necessary ports only
   - Consider running on private networks/VPNs for sensitive use cases
   - Monitor network traffic for unusual patterns

3. **Container Security**

   ```bash
   # Limit container capabilities
   docker run --cap-drop=ALL --cap-add=NET_BIND_SERVICE w4ff1e/onvif-media-transcoder:latest
   ```

4. **Monitoring and Logging**
   - Enable container logging and monitoring
   - Watch for authentication failures and unusual access patterns

### Development Security

1. **Dependency Security**
   - Regularly update Rust dependencies
   - Review dependency licenses and security advisories

2. **Code Review**
   - All changes should be reviewed for security implications
   - Pay special attention to authentication and network code
   - Validate AI-generated code components thoroughly

3. **Testing**
   - Include security testing in CI/CD pipeline
   - Test authentication mechanisms with various clients
   - Validate input sanitization and error handling

## Security Resources

- [ONVIF Security Guidelines](https://www.onvif.org/specs/guidelines.html)
- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [NIST Container Security Guide](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-190.pdf)

## Comments on this Policy

If you have suggestions on how this security policy could be improved, please:

1. Submit a pull request with proposed changes
2. Open an issue for discussion
3. Contact the maintainers directly

We appreciate community feedback to enhance the security posture of this project.

---

**Last Updated**: July 2025  
**Policy Version**: 2.0
